use std::{error::Error, fmt};
use stdweb::web::Element as DOMElement;
use stdweb::web::{document, IElement, INode, INonElementParentNode};

/// Represents the current state of the user interface.
pub enum State {
    /// The user interface is presenting a restaurant for the user's consideration.
    Presenting,
    /// The app has run out of suggestions and is shrugging at the user.
    Terminated,
}

/// Represents a uniquely identifiable HTML element.
///
/// Utilizes the
/// [`id` global attribute](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/id)
/// under the hood.
#[derive(Clone, Copy, Debug)]
struct Element(&'static str);

impl Element {
    /// Returns the associated `id` (no leading `#`).
    fn id(&self) -> &str {
        self.0
    }

    /// Returns the element uniquely associated with the appropriate `id`, if it exists.
    fn get(&self) -> Option<DOMElement> {
        document().get_element_by_id(self.0)
    }

    /// Sets both a glyph (emoji or other character) and an accessible alternate text for a given
    /// element.
    ///
    /// # Notes
    /// Use this method to set emoji labels.
    fn set_glyph(&self, new: &str, alt: &str) -> Result<(), GetElementError> {
        self.get()
            .map(|e| {
                e.set_text_content(new);
                let _ = e.set_attribute("aria-label", alt);
            })
            .ok_or(self.error())
    }

    /// Provides an interface with which to set plain text content for a given element.
    ///
    /// # Notes
    /// When using emoji, use `set_glyph` instead. This may become a hard error in the future.
    fn set_text(&self, new: &str) -> Result<(), GetElementError> {
        self.get()
            .map(|e| {
                e.set_text_content(new);
                e.remove_attribute("aria-label");
            })
            .ok_or(self.error())
    }

    /// Set the `data-{name}` attribute of the element to `value`.
    ///
    /// Useful for storing state information in the DOM.
    fn set_data_attribute(&self, name: &str, value: &str) -> Result<(), GetElementError> {
        self.get()
            .map(|e| e.set_attribute(&format!("data-{}", name), value).unwrap())
            .ok_or(self.error())
    }

    /// Removes the `data-{name}` attribute from the element.
    fn clear_data_attribute(&self, name: &str) -> Result<(), GetElementError> {
        self.get()
            .map(|e| e.remove_attribute(&format!("data-{}", name)))
            .ok_or(self.error())
    }

    /// Returns whether the `data-{name}` attribute exists on the element.
    fn has_data_attribute(&self, name: &str) -> Result<bool, GetElementError> {
        self.get()
            .map(|e| e.has_attribute(&format!("data-{}", name)))
            .ok_or(self.error())
    }

    /// Returns the error associated with the inability to fetch this element from the DOM.
    fn error(self) -> GetElementError {
        GetElementError::new(self)
    }
}

/// An internal error representing a failure to fetch a particular element.
///
/// This error typically occurs if the requested element does not exist in the DOM.
#[derive(Debug)]
struct GetElementError {
    element: Element,
}

impl GetElementError {
    /// Creates an error describing a generic failure to fetch the given element.
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

/// Sets the state of the application user interface.
///
/// The current application state is stored in the DOM.
///
/// # Errors
/// This method returns `Err(impl Error)` if an error occurs while updating (or fetching)
/// any elements.
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

/// Returns the state of the application user interface.
///
/// The current application state is stored in the DOM.
pub fn get_state() -> Result<State, impl Error> {
    Element("next").has_data_attribute("terminated").map(|b| {
        if b {
            State::Terminated
        } else {
            State::Presenting
        }
    })
}

/// Updates the application user interface to reflect the new suggestion.
pub fn set_suggestion(name: &str, hours: &str) -> Result<(), impl Error> {
    Element("place").set_text(&name)?;
    Element("times").set_text(&hours)
}

/// Shows the "next" button, which is hidden by default.
///
/// Invoked in the `start()` method, when we know script execution works.
pub fn unhide_button() {
    // We can't currently change the style of an element with stdnet,
    // so call into JavaScript to unhide the button.
    js! {
        document.getElementById("next").style.display = "initial";
    }
}
