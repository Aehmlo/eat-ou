#[macro_use]
extern crate stdweb;

extern crate eat_ou;

use eat_ou::*;
use stdweb::{
    unstable::TryInto,
    web::{
        document, event::{ClickEvent, IKeyboardEvent, KeyUpEvent}, Date, IEventTarget,
    },
};

mod ui;

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
    // TODO: Remove clone if possible
    let callback = create_callback(restaurants.clone());
    // stdweb doesn't support the options argument to addEventListener, so use JavaScript
    js! {
        document.getElementById("next").addEventListener("click", @{callback}, { once: true });
    }
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
        match ui::get_state() {
            Ok(ui::State::Terminated) => start(),
            _ => {
                end();
                add_event_listener(restaurants);
            }
        }
    }
}

fn suggest(restaurant: Restaurant) {
    match restaurant.get_hours(today()) {
        Some(hours) => ui::set_suggestion(&restaurant.name, &format!("{}", hours)).unwrap(),
        None => ui::set_suggestion(&restaurant.name, &"").unwrap(),
    }
}

fn start() {
    let mut restaurants = get_viable();
    shuffle(&mut restaurants);
    ui::set_state(ui::State::Presenting).unwrap();
    next(&mut restaurants);
}

fn end() {
    ui::set_state(ui::State::Terminated).unwrap();
}

fn bind_spacebar() {
    document().add_event_listener::<KeyUpEvent, _>(move |event| {
        if event.key() == " " {
            // stdweb doesn't yet support click(), so use JavaScript
            js! {
                document.getElementById("next").click();
            }
        }
    });
}

fn main() {
    stdweb::initialize();
    ui::unhide_button();
    start();
    bind_spacebar();
    stdweb::event_loop();
}
