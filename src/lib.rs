//! This is a module-level comment for a Rust lib

#![deny(clippy::arithmetic_side_effects)]
#![cfg_attr(not(test), deny(unused_crate_dependencies))]

use tokio as _;

mod command;

pub use command::*;
