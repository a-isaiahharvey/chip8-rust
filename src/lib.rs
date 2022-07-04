pub mod app;
pub mod cpu;
pub mod instruction;
pub mod register;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
