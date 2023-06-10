// #![allow(dead_code)]
#![warn(clippy::cast_lossless)]
#![warn(clippy::uninlined_format_args)]
#![warn(clippy::semicolon_if_nothing_returned)]
//
// #![deny(clippy::pedantic)]
// #![allow(clippy::similar_names)]
// #![allow(clippy::excessive_precision)]
// #![allow(clippy::unreadable_literal)]
// #![allow(clippy::wildcard_imports)]
// #![allow(clippy::too_many_lines)]
// #![allow(clippy::missing_panics_doc)]
// #![allow(clippy::missing_errors_doc)]
// #![allow(clippy::must_use_candidate)]
// #![allow(clippy::enum_glob_use)]
// #![allow(clippy::cast_precision_loss)]
// #![allow(clippy::cast_possible_truncation)]

pub mod audio;
pub mod defs;
pub mod device;
pub mod dsp;
pub mod effect;
pub mod instrument;
pub mod lua;
pub mod render;
pub mod scope;
