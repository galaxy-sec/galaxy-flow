mod conf_utils;
//mod delegate;
//mod manager;
mod toml_impl;
mod traits;

pub use conf_utils::{backup_clean, clear_file, read_file, save_data};
pub use traits::ConfigLifecycle;
