use std::{error::Error, fmt};
use stdweb::web::Element as DOMElement;
use stdweb::web::{document, IElement, INode, INonElementParentNode};

pub enum State {
    Presenting,
    Terminated,
}

#[derive(Clone, Copy, Debug)]
struct Element(&'static str);

impl Element {
    fn id(&self) -> &str {
        self.0
    }

    fn get(&self) -> Option<DOMElement> {
        document().get_element_by_id(self.0)
    }

    fn set_glyph(&self, new: &str, alt: &str) -> Result<(), GetElementError> {
        self.get()
            .map(|e| {
                e.set_text_content(new);
                let _ = e.set_attribute("aria-label", alt);
            })
            .ok_or(self.error())
    }

    fn set_text(&self, new: &str) -> Result<(), GetElementError> {
        self.get()
            .map(|e| {
                e.set_text_content(new);
                e.remove_attribute("aria-label");
            })
            .ok_or(self.error())
    }

    fn set_data_attribute(&self, name: &str, value: &str) -> Result<(), GetElementError> {
        self.get()
            .map(|e| e.set_attribute(&format!("data-{}", name), value).unwrap())
            .ok_or(self.error())
    }

    fn clear_data_attribute(&self, name: &str) -> Result<(), GetElementError> {
        self.get()
            .map(|e| e.remove_attribute(&format!("data-{}", name)))
            .ok_or(self.error())
    }

    fn has_data_attribute(&self, name: &str) -> Result<bool, GetElementError> {
        self.get()
            .map(|e| e.has_attribute(&format!("data-{}", name)))
            .ok_or(self.error())
    }

    fn error(self) -> GetElementError {
        GetElementError::new(self)
    }
}

#[derive(Debug)]
struct GetElementError {
    element: Element,
}

impl GetElementError {
    fn new(element: Element) -> Self {
        Self { element }
    }
}

impl fmt::Display for GetElementError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "Failed to fetch element with ID \"{}\"",
            self.element.id()
        )
    }
}

impl Error for GetElementError {}

pub fn set_state(state: State) -> Result<(), impl Error> {
    let next_text = Element("next_text");
    let place = Element("place");
    let times = Element("times");
    let next_button = Element("next");
    match state {
        State::Terminated => {
            next_text.set_glyph("🔄", "Start over")?;
            place.set_glyph("🤷", "Out of suggestions")?;
            times.set_text("There aren't any places left to eat. Try again?")?;
            next_button.set_data_attribute("terminated", "1")
        }
        State::Presenting => {
            next_text.set_glyph("👎", "Next suggestion")?;
            place.set_text("")?;
            times.set_text("")?;
            next_button.clear_data_attribute("terminated")
        }
    }
}

pub fn get_state() -> Result<State, impl Error> {
    Element("next").has_data_attribute("terminated").map(|b| {
        if b {
            State::Terminated
        } else {
            State::Presenting
        }
    })
}

pub fn set_suggestion(name: &str, hours: &str) -> Result<(), impl Error> {
    Element("place").set_text(&name)?;
    Element("times").set_text(&hours)
}

pub fn unhide_button() {
    // We can't currently change the style of an element with stdnet,
    // so call into JavaScript to unhide the button.
    js! {
        document.getElementById("next").style.display = "initial";
    }
}
