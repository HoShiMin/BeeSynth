#![warn(clippy::pedantic)]

pub mod interface;
pub mod loader;
pub mod inpout_impl;
pub mod error;


mod slicer;

use winapi::auto;

pub use interface::{Interface, PhysMapping};

pub struct Inpout {
    device_handle: auto::FileHandle
}

unsafe impl Send for Inpout {}
unsafe impl Sync for Inpout {}