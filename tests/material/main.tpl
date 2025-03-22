[package]
name    = "rg-engine"
version = "{{VERSION}}"
authors = ["wukong <sec-wukong@outlook.com>"]
edition = "2018"



[dependencies]
nom                = "5.0"
log                = "0.4.1"
serde              = "1.0"
serde_derive       = "1.0"
serde_json         = "1.0"
lazy_static        = "1.0"
futures            = "0.1"
pretty_env_logger  = "0.2"
toml               = "0.4"
err-derive         = "0.2"
shells             = "0.2"
derive_builder     = "0.9"
derive-getters     = "0.1"
once_cell          = "1.2"
mockall            = "0.6"
handlebars         = "2.0"
clap               = "2.31"
regex              = "1.3"
async-trait        = "0.1"
async-std          = "1.5"
async-h1           = "1.0"
http-types         = "1.0"
trust-dns-resolver = "0.19"
url                = "2.1"

{{{rg_lib1}}}

{{{rg_lib2}}}

[features]
default    = ["nightly"]
nightly    = ["nightly_rg-lib"]
beta       = ["beta_rg-lib"]
master     = ["master_rg-lib"]
release    = ["release_rg-lib"]
