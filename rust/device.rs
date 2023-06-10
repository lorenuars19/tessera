// can autogen this

use crate::effect::Effect;
use crate::effect::{gain::Gain, pan::Pan};
use crate::instrument::Instrument;
use crate::instrument::{sine::Sine, wavetable::Wavetable};

// list of instruments
pub fn new_instrument(sample_rate: f32, instrument_number: usize) -> Box<dyn Instrument + Send> {
	match instrument_number {
		0 => Box::new(Sine::new(sample_rate)),
		1 => Box::new(Wavetable::new(sample_rate)),
		_ => {
			eprintln!("Instrument with number {instrument_number} not found. Returning default.");
			Box::new(Sine::new(sample_rate))
		}
	}
}

// list of effects
pub fn new_effect(sample_rate: f32, effect_number: usize) -> Box<dyn Effect + Send> {
	match effect_number {
		0 => Box::new(Pan::new(sample_rate)),
		1 => Box::new(Gain::new(sample_rate)),
		_ => {
			eprintln!("Effect with number {effect_number} not found. Returning default.");
			Box::new(Gain::new(sample_rate))
		}
	}
}

pub trait Param {
	fn set_param(&mut self, index: usize, val: f32);
}
