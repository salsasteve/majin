#![cfg_attr(not(feature = "debug"), no_std)]
extern crate alloc;

pub mod core;

#[cfg(feature = "debug")]
extern crate std;
