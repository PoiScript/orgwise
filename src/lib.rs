#![allow(async_fn_in_trait)]
#![allow(dead_code)]

mod backend;
#[cfg(feature = "tower")]
mod cli;
mod command;
mod lsp;
#[cfg(test)]
mod test;
mod utils;
#[cfg(feature = "wasm")]
mod wasm;
