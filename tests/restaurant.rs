#[macro_use]
extern crate serde_json;

extern crate eat_ou;

use eat_ou::Restaurant;

#[test]
fn test_json() {
    let _: Vec<Restaurant> = serde_json::from_str(include_str!("../food.json")).unwrap();
}
