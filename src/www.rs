#[macro_use]
extern crate stdweb;

extern crate eat_ou;

use eat_ou::*;
use stdweb::{
    unstable::TryInto,
    web::{
        document, event::{ClickEvent, IKeyboardEvent, KeyDownEvent}, Date, IEventTarget, INode,
        INonElementParentNode,
    },
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

fn today() -> Day {
    Date::new().get_day().into()
}

fn add_event_listener(restaurants: &mut Vec<Restaurant>) {
    let callback = create_callback(restaurants.clone()); // TODO: Remove clone if possible
    document()
        .get_element_by_id("next")
        .unwrap()
        .add_event_listener::<ClickEvent, _>(callback);
}

fn create_callback(mut restaurants: Vec<Restaurant>) -> impl FnMut(ClickEvent) {
    move |_| {
        next(&mut restaurants);
    }
}

fn next(restaurants: &mut Vec<Restaurant>) {
    if let Some(restaurant) = restaurants.pop() {
        suggest(restaurant);
        add_event_listener(restaurants);
    } else {
        end();
    }
}

fn suggest(restaurant: Restaurant) {
    if let Some(hours) = restaurant.get_hours(today()) {
        document()
            .get_element_by_id("times")
            .unwrap()
            .set_text_content(&format!("{}", hours));
    }
    let name = restaurant.name;
    document()
        .get_element_by_id("place")
        .unwrap()
        .set_text_content(&name);
}

fn start() {
    let mut restaurants = get_viable();
    shuffle(&mut restaurants);
    next(&mut restaurants);
    document().add_event_listener::<KeyDownEvent, _>(move |event| {
        if event.key() == " " {
            // stdweb doesn't yet support click(), so we'll use JavaScript
            js! {
                document.getElementById("next").click();
            }
        }
    });
}

fn end() {}

fn main() {
    stdweb::initialize();
    // We can't currently change the style of an element from within Rust,
    // so call into JavaScript to unhide the button.
    js! {
        document.getElementById("next").style.display = "initial";
    }
    start();
    stdweb::event_loop();
}
