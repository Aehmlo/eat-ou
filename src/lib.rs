#![feature(extern_prelude)]
#![feature(exclusive_range_pattern)]

#[macro_use]
extern crate serde_derive;
extern crate serde;

#[macro_use]
extern crate stdweb;

mod schedule;
pub use schedule::{Day, Restaurant, Time};

/// Manages the application user interface.
pub mod ui;

extern crate serde_json;

#[test]
fn test_json() {
    let _: Vec<Restaurant> = serde_json::from_str(include_str!("../food.json")).unwrap();
}
