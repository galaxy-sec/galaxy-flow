#[macro_use]
extern crate log;
extern crate pretty_env_logger;

extern crate regex;
#[allow(unused_imports)]
#[macro_use]
extern crate duct;
extern crate duct_sh;
#[macro_use]
extern crate derive_getters;
#[macro_use]
extern crate mockall;
extern crate once_cell;
extern crate shells;
extern crate toml;
#[macro_use]
extern crate derive_builder;
extern crate handlebars;
// extern crate json ;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
extern crate http_types;

extern crate ini;
extern crate url;

extern crate trust_dns_resolver;
//#[macro_use]
//extern crate simple_log;

#[macro_use]
pub mod err;
#[macro_use]
pub mod model;

pub mod ability;
pub mod calculate;
pub mod debug;
mod evaluator;
mod loader;
pub mod menu;
pub mod parser;
#[macro_use]
pub mod util;

pub mod infra;
pub mod runner;
pub mod types;

pub use crate::loader::{get_parse_code, GxLoader};
pub use model::*;
