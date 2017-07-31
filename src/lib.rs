#![feature(plugin)]
#![plugin(rocket_codegen)]

#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate rocket;
extern crate uuid;

#[macro_use]
extern crate serde_derive;

pub mod api;
pub mod cqrs;
pub mod domain;
