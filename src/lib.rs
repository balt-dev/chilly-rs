#![warn(missing_docs, clippy::pedantic, clippy::perf)]
#![doc = include_str!(r"../README.md")]

extern crate core;

pub mod parser;
pub mod structures;
pub mod database;
