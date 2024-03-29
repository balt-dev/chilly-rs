#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic, clippy::perf)]
#![doc = include_str!(r"../README.md")]

pub mod parser;
pub mod structures;
pub mod database;
pub mod arguments;
pub mod solidify;
pub mod renderer;

// TODO: Re-exports
