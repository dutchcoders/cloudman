extern crate cursive;

use cursive::direction::Direction;
use cursive::event::{Callback, Event, EventResult, Key};
use cursive::view::View;
use cursive::{Cursive, Printer};
use std::rc::Rc;

pub type OnEdit = dyn Fn(&mut Cursive, &str, usize);
pub type OnClose = dyn Fn(&mut Cursive);

#[derive(Default)]
pub struct Foo {
    info: Rc<String>,
    on_search: Option<Rc<OnEdit>>,
    on_search_next: Option<Rc<OnEdit>>,
    on_cancel: Option<Rc<OnClose>>,
    on_close: Option<Rc<OnClose>>,
}

impl Foo {
    pub fn with_string(s: &str) -> Self {
        Foo {
            info: Rc::new(String::from(s)),

            on_search: None,
            on_search_next: None,
            on_cancel: None,
            on_close: None,
        }
    }

    pub fn set_on_search<F>(&mut self, callback: F)
    where
        F: Fn(&mut Cursive, &str, usize) + 'static,
    {
        self.on_search = Some(Rc::new(callback));
    }

    pub fn set_on_search_next<F>(&mut self, callback: F)
    where
        F: Fn(&mut Cursive, &str, usize) + 'static,
    {
        self.on_search_next = Some(Rc::new(callback));
    }

    pub fn set_on_cancel<F>(&mut self, callback: F)
    where
        F: Fn(&mut Cursive) + 'static,
    {
        self.on_cancel = Some(Rc::new(callback));
    }

    pub fn set_on_close<F>(&mut self, callback: F)
    where
        F: Fn(&mut Cursive) + 'static,
    {
        self.on_close = Some(Rc::new(callback));
    }

    fn make_cancel_cb(&self) -> Option<Callback> {
        self.on_cancel.clone().map(|cb| {
            // Get a new Rc on the content
            Callback::from_fn(move |s| {
                cb(s);
            })
        })
    }

    fn make_close_cb(&self) -> Option<Callback> {
        self.on_close.clone().map(|cb| {
            // Get a new Rc on the content
            Callback::from_fn(move |s| {
                cb(s);
            })
        })
    }

    fn make_search_next_cb(&self) -> Option<Callback> {
        self.on_search_next.clone().map(|cb| {
            // Get a new Rc on the content
            let content = Rc::clone(&self.info);
            let cursor = 0;

            Callback::from_fn(move |s| {
                cb(s, &content, cursor);
            })
        })
    }

    fn make_search_cb(&self) -> Option<Callback> {
        self.on_search.clone().map(|cb| {
            // Get a new Rc on the content
            let content = Rc::clone(&self.info);
            let cursor = 0;

            Callback::from_fn(move |s| {
                cb(s, &content, cursor);
            })
        })
    }

    pub fn delete(&mut self) -> Callback {
        Rc::make_mut(&mut self.info).pop();
        self.make_search_cb().unwrap_or_else(Callback::dummy)
    }

    pub fn insert(&mut self, ch: char) -> Callback {
        Rc::make_mut(&mut self.info).push(ch);
        self.make_search_cb().unwrap_or_else(Callback::dummy)
    }
}

impl View for Foo {
    fn draw(&self, _: &Printer) {
        // printer.print((0, 0), &self.info);
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        true
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Key(Key::Backspace) => EventResult::Consumed(Some(self.delete())),
            Event::Key(Key::Enter) => EventResult::Consumed(Some(self.make_close_cb().unwrap())),
            Event::Key(Key::Esc) => EventResult::Consumed(Some(self.make_cancel_cb().unwrap())),
            Event::Key(Key::F3) => EventResult::Consumed(Some(self.make_search_next_cb().unwrap())),
            Event::Char(ch) => EventResult::Consumed(Some(self.insert(ch))),
            _ => EventResult::Consumed(Some(self.make_close_cb().unwrap())),
        }
    }
}
