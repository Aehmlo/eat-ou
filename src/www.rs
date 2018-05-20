#[macro_use]
extern crate stdweb;

extern crate eat_ou;

use eat_ou::*;
use stdweb::{
    unstable::TryInto,
    web::{
        document, event::{ClickEvent, IKeyboardEvent, KeyUpEvent}, Date, IEventTarget,
        INonElementParentNode,
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

fn tuplify(vec: Vec<Restaurant>) -> Vec<(String, String, bool)> {
    let mut viable = vec.iter()
        .filter(|r| r.is_viable(today(), now()))
        .collect::<Vec<_>>();
    let mut not = vec.iter()
        .filter(|r| !r.is_viable(today(), now()))
        .collect::<Vec<_>>();
    viable.sort_by_key(|r| r.name.clone());
    not.sort_by_key(|r| r.name.clone());
    viable.append(&mut not);
    let vec = viable;
    vec.iter()
        .map(|r| {
            (
                r.name.clone(),
                format!("{}", r.get_hours(today()).unwrap()),
                r.is_viable(today(), now()),
            )
        })
        .collect::<Vec<_>>()
}

/// Binds event listener to the "next" button.
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
    match ui::get_state() {
        Ok(ui::State::Presenting) | Ok(ui::State::Terminated) => {
            if let Some(restaurant) = restaurants.pop() {
                suggest(restaurant);
                add_event_listener(restaurants);
            } else {
                if ui::get_state().unwrap() == ui::State::Terminated {
                    start()
                } else {
                    end();
                    add_event_listener(restaurants);
                }
            }
        }
        _ => add_event_listener(restaurants), // Re-bind event listener
    };
}

fn list() {
    let restaurants = get_viable(); // Restaurant::get_list()
    ui::tabulate(tuplify(restaurants));
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
/// Also binds an event listener to the l key, forwarding keyup events to the list button.
/// stdweb doesn't yet support click(), so we use JavaScript.
fn bind_keyboard() {
    document().add_event_listener::<KeyUpEvent, _>(move |event| match event.key().as_str() {
        " " => {
            js! { document.getElementById("next").click(); };
        }
        "l" => {
            js! { document.getElementById("list").click(); };
        }
        _ => {}
    });
}

fn toggle_list_mode() {
    match ui::get_state() {
        Ok(ui::State::Terminated) | Ok(ui::State::Presenting) => {
            list();
        }
        Ok(ui::State::Tabulating) => {
            ui::stop_tabulation();
        }
        Err(_) => {} // TODO: Handle error
    };
}

/// Binds an event listener to the list button, enabling the button to switch view modes.
fn bind_list() {
    document()
        .get_element_by_id("list")
        .unwrap()
        .add_event_listener::<ClickEvent, _>(|_| {
            toggle_list_mode();
        });
}

fn main() {
    stdweb::initialize();
    ui::unhide_button();
    start();
    bind_keyboard();
    bind_list();
    stdweb::event_loop();
}
