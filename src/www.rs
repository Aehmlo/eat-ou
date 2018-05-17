#[macro_use]
extern crate stdweb;

extern crate eat_ou;

use eat_ou::*;
use stdweb::{
    unstable::TryInto, web::{document, Date, INode, INonElementParentNode},
};

fn get_viable() -> Vec<Restaurant> {
    let now = Date::new();
    let day: Day = now.get_day().into();
    let (hours, minutes) = (now.get_hours(), now.get_minutes());
    let time = Time::new(hours, minutes);
    Restaurant::get_list()
        .into_iter()
        .filter(|r| r.is_viable(day, time))
        .collect()
}

/// Performs an in-place naïve Fisher-Yates shuffle.
fn shuffle<T>(vec: &mut Vec<T>) {
    let len = vec.len() as u32;

    for i in 0..len {
        let j = len - i;
        // Use JavaScript's Math.random() instead of using the rand crate,
        // due to current limitations.
        let index: u32 = js!{ return Math.floor(Math.random() * @{j}); }
            .try_into()
            .unwrap();
        vec.swap(index as usize, (j - 1) as usize);
    }
}

fn main() {
    stdweb::initialize();
    let mut restaurants = get_viable();
    shuffle(&mut restaurants);
    let name = restaurants[0].name.to_owned();
    document().get_element_by_id("place").unwrap().set_text_content(&name);
    // We can't currently change the style of an element from within Rust,
    // so call into JavaScript to unhide the button.
    js! {
        document.getElementById("next").style.display = "initial";
    }
    stdweb::event_loop();
}
