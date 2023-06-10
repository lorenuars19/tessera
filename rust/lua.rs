use crate::audio;
use crate::defs::*;
use crate::render;
use crate::scope::Scope;
use mlua::prelude::*;
use mlua::{UserData, UserDataMethods, Value};
use ringbuf::{Consumer, Producer};
use std::sync::{Arc, Mutex};

struct LuaData(Option<AudioContext>);

pub struct AudioContext {
	pub stream: cpal::Stream,
	pub audio_tx: Producer<AudioMessage>,
	pub stream_tx: Producer<bool>,
	pub lua_rx: Consumer<LuaMessage>,
	pub m_render: Arc<Mutex<render::Render>>,
	pub scope: Scope,
	pub paused: bool,
}

impl Drop for AudioContext {
	fn drop(&mut self) {
		println!("Stream dropped");
	}
}

fn init(_: &Lua, _: ()) -> LuaResult<LuaData> {
	Ok(LuaData(None))
}

impl UserData for LuaData {
	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method_mut(
			"setup",
			|_, data, (host_name, device_name): (String, String)| match audio::run(
				&host_name,
				&device_name,
			) {
				Ok(ud) => {
					*data = LuaData(Some(ud));
					Ok(())
				}
				Err(e) => {
					println!("{e}");
					*data = LuaData(None);
					Ok(())
				}
			},
		);

		methods.add_method_mut("quit", |_, data, _: ()| {
			*data = LuaData(None);
			Ok(())
		});

		methods.add_method_mut("set_working_directory", |_, _, path: String| {
			std::env::set_current_dir(std::path::Path::new(&path))?;
			Ok(())
		});

		methods.add_method("running", |_, LuaData(ud), _: ()| Ok(ud.is_some()));

		methods.add_method_mut("send_cv", |_, data, (ch, pitch, vel): (usize, f32, f32)| {
			if let LuaData(Some(ud)) = data {
				ud.send_message(AudioMessage::CV(ch, pitch, vel));
			}
			Ok(())
		});

		methods.add_method_mut(
			"send_note_on",
			|_, data, (ch, pitch, vel, id): (usize, f32, f32, usize)| {
				if let LuaData(Some(ud)) = data {
					ud.send_message(AudioMessage::Note(ch, pitch, vel, id));
				}
				Ok(())
			},
		);

		methods.add_method_mut("send_mute", |_, data, (ch, mute): (usize, bool)| {
			if let LuaData(Some(ud)) = data {
				ud.send_message(AudioMessage::Mute(ch, mute));
			}
			Ok(())
		});

		methods.add_method_mut(
			"send_param",
			|_, data, (ch_index, device_index, index, value): (usize, usize, usize, f32)| {
				if let LuaData(Some(ud)) = data {
					ud.send_message(AudioMessage::SetParam(ch_index, device_index, index, value));
				}
				Ok(())
			},
		);

		methods.add_method_mut("set_paused", |_, data, paused: bool| {
			if let LuaData(Some(ud)) = data {
				ud.send_paused(paused);
			}
			Ok(())
		});

		methods.add_method("paused", |_, data, _: ()| {
			if let LuaData(Some(ud)) = data {
				Ok(ud.paused)
			} else {
				Ok(true)
			}
		});

		methods.add_method("add_channel", |_, data, instrument_number: usize| {
			if let LuaData(Some(ud)) = data {
				let mut render = ud.m_render.lock().expect("Failed to get lock.");
				render.add_channel(instrument_number);
			}
			Ok(())
		});

		methods.add_method(
			"add_effect",
			|_, data, (channel_index, effect_number): (usize, usize)| {
				if let LuaData(Some(ud)) = data {
					let mut render = ud.m_render.lock().expect("Failed to get lock.");
					render.add_effect(channel_index, effect_number);
				}
				Ok(())
			},
		);

		methods.add_method("render_block", |_, data, _: ()| {
			if let LuaData(Some(ud)) = data {
				let len = 64;
				let buffer: &mut [&mut [f32]; 2] = &mut [&mut vec![0.0; len], &mut vec![0.0; len]];
				let mut out_buffer = vec![0.0f64; len * 2];

				let mut render = ud.m_render.lock().expect("Failed to get lock.");
				// TODO: need to check here if the stream is *actually* paused

				render.parse_messages();
				render.process(buffer);
				// interlace and convert to i16 as f64 (lua wants doubles anyway)
				for (i, outsample) in out_buffer.chunks_exact_mut(2).enumerate() {
					outsample[0] = convert_sample_wav(buffer[0][i]);
					outsample[1] = convert_sample_wav(buffer[1][i]);
				}
				Ok(Some(out_buffer))
			} else {
				Ok(None)
			}
		});

		methods.add_method_mut("update_scope", |_, data, _: ()| {
			if let LuaData(Some(ud)) = data {
				ud.scope.update();
			}
			Ok(())
		});
		methods.add_method("get_spectrum", |_, data, _: ()| {
			if let LuaData(Some(ud)) = data {
				Ok(Some(ud.scope.get_spectrum()))
			} else {
				Ok(None)
			}
		});
		methods.add_method("get_scope", |_, data, _: ()| {
			if let LuaData(Some(ud)) = data {
				Ok(Some(ud.scope.get_oscilloscope()))
			} else {
				Ok(None)
			}
		});

		methods.add_method_mut("rx_pop", |_, data, _: ()| {
			if let LuaData(Some(ud)) = data {
				Ok(ud.lua_rx.pop())
			} else {
				Ok(None)
			}
		});
	}
}

#[mlua::lua_module]
fn rust_backend(lua: &Lua) -> LuaResult<LuaTable> {
	let exports = lua.create_table()?;
	exports.set("init", lua.create_function(init)?)?;
	Ok(exports)
}

#[inline]
fn dither() -> f64 {
	// Don't know if this is the correct scaling for dithering, but it sounds good
	(fastrand::f64() - fastrand::f64()) / (2.0 * f64::from(i16::MAX))
}

#[inline]
fn convert_sample_wav(x: f32) -> f64 {
	let z = (f64::from(x) + dither()).clamp(-1.0, 1.0);

	if z >= 0.0 {
		z * f64::from(i16::MAX)
	} else {
		-z * f64::from(i16::MIN)
	}
}

impl AudioContext {
	fn send_message(&mut self, m: AudioMessage) {
		if self.audio_tx.push(m).is_err() {
			println!("Queue full. Dropped message!");
		}
	}

	fn send_paused(&mut self, paused: bool) {
		self.paused = paused;
		if self.stream_tx.push(paused).is_err() {
			println!("Stream queue full. Dropped message!");
		}
	}
}

impl<'lua> ToLua<'lua> for LuaMessage {
	fn to_lua(self, lua: &'lua Lua) -> LuaResult<Value<'lua>> {
		use LuaMessage::*;

		let table = Lua::create_table(lua)?;

		match self {
			Cpu(cpu_load) => {
				table.set("tag", "cpu")?;
				table.set("cpu_load", cpu_load)?;
			}
			Meter(l, r) => {
				table.set("tag", "meter")?;
				table.set("l", l)?;
				table.set("r", r)?;
			}
		}

		Ok(Value::Table(table))
	}
}
