#![feature(plugin)]

#![plugin(rocket_codegen)]

extern crate rocket;
extern crate uuid;

#[macro_use]
extern crate serde_derive;

pub mod api;
pub mod cqrs;
pub mod domain;
