#![no_std]

pub(crate) mod fmt;

pub mod sniffer;
pub mod guesser;

pub use guesser::{OS, OSGuesser};
