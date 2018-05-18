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

/// Returns the current day as an instance of `Day`.
///
/// Depends on JavaScript APIs for time information.
fn today() -> Day {
    Date::new().get_day().into()
}

/// Returns the approximate current time as an instance of `Time`.
///
/// Depends on JavaScript APIs for time information.
fn now() -> Time {
    let now = Date::new();
    Time::new(now.get_hours(), now.get_minutes())
}
/// Get viable restaurants based on the user's local time.
///
/// Depends on JavaScript APIs for time information.
fn get_viable() -> Vec<Restaurant> {
    Restaurant::get_list()
        .into_iter()
        .filter(|r| r.is_viable(today(), now()))
        .collect()
}

/// Performs an in-place naïve Fisher-Yates shuffle.
///
/// Depends on JavaScript APIs for random number generation.
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

/// Binds an event listener to the "next" button.
///
/// The associated callback forwards the invocation to the `next` function.
///
/// Depends on JavaScript APIs to attach a single-use event listener.
fn add_event_listener(restaurants: &mut Vec<Restaurant>) {
    // TODO: Remove clone if possible
    let mut restaurants = restaurants.clone();
    let callback = move |_: ClickEvent| {
        next(&mut restaurants);
    };
    // stdweb doesn't support the options argument to addEventListener, so use JavaScript
    js! {
        document.getElementById("next").addEventListener("click", @{callback}, { once: true });
    }
}

/// Progresses to the next restaurant recommendation.
///
/// If there are no more restaurants, progresses to the end state.
/// If already in the end state, calls `start` and begins the cycle anew.
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

/// Presents a restaurant for the user's consideration.
fn suggest(restaurant: Restaurant) {
    match restaurant.get_hours(today()) {
        Some(hours) => ui::set_suggestion(&restaurant.name, &format!("{}", hours)).unwrap(),
        None => ui::set_suggestion(&restaurant.name, &"").unwrap(),
    }
}

/// Starts the suggestion cycle, generating and shuffling a new list of restaurants.
///
/// Calls `next` to begin presenting options.
fn start() {
    let mut restaurants = get_viable();
    shuffle(&mut restaurants);
    ui::set_state(ui::State::Presenting).unwrap();
    next(&mut restaurants);
}

/// Stops the suggestion cycle, presenting the end screen.
fn end() {
    ui::set_state(ui::State::Terminated).unwrap();
}

/// Binds an event listener to the spacebar, forwarding keyup events to the next button.
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
