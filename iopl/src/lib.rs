#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod level;

#[cfg(target_os = "windows")]
pub mod windows;
