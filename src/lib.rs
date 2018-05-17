#![feature(extern_prelude)]

#[macro_use]
extern crate serde_derive;
extern crate serde;

mod schedule;
pub use schedule::{Day, Restaurant, Time};

extern crate serde_json;

#[test]
fn test_json() {
    let _: Vec<Restaurant> = serde_json::from_str(include_str!("../food.json")).unwrap();
}
