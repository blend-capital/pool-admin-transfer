#![no_std]

#[cfg(test)]
extern crate std;

mod contract;
mod errors;
mod storage;

pub use contract::*;

#[cfg(test)]
mod test;

#[cfg(test)]
pub mod testutils;
