extern crate cursive;

use cursive::direction::Direction;
use cursive::event::{Event, EventResult, Key};
use cursive::vec::Vec2;
use cursive::view::{ScrollBase, View};
use cursive::Printer;

pub struct LogView {
    lines: Vec<String>,

    scrollbase: ScrollBase,
}

impl LogView {
    /// Create a new `FlexiLoggerView` which is wrapped in a `ScrollView`.
    pub fn scrollable(buf: &[u8]) -> Self {
        let lines = parse_lines(&buf);

        LogView {
            lines,
            scrollbase: ScrollBase::new().right_padding(0),
        }
    }
}

fn parse_lines(buf: &[u8]) -> Vec<String> {
    let mut statemachine = vte::Parser::new();
    let mut parser = Log::new();

    for byte in &buf[..] {
        statemachine.advance(&mut parser, *byte);
    }

    parser.lines
}

impl View for LogView {
    fn draw(&self, printer: &Printer<'_, '_>) {
        self.scrollbase.draw(printer, |printer, i| {
            let lines = self.lines.clone();

            // ignore the first line, as it is incomplete
            if let Some(line) = lines.get(i+1) {
                printer.print((0, 0), line);
            } else {
                printer.print((0, 0), "⍇");
            }
        });
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        let lines = self.lines.clone();

        let h = std::cmp::max(lines.len(), constraint.y);

        self.scrollbase.set_heights(constraint.y, h+1);

        constraint
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        true
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        let height = self.scrollbase.view_height;
        match event {
            Event::Key(Key::Up) => {
                self.scrollbase.scroll_up(1);
                EventResult::Consumed(None)
            }
            Event::Key(Key::Down) => {
                self.scrollbase.scroll_down(1);
                EventResult::Consumed(None)
            }
            Event::Key(Key::PageUp) => {
                self.scrollbase.scroll_up(height);
                EventResult::Consumed(None)
            }
            Event::Key(Key::PageDown) => {
                self.scrollbase.scroll_down(height);
                EventResult::Consumed(None)
            }
            Event::Char('g') => {
                self.scrollbase.scroll_top();
                EventResult::Consumed(None)
            }
            Event::Shift(Key::Home) => {
                self.scrollbase.scroll_top();
                EventResult::Consumed(None)
            }
            Event::Shift(Key::End) => {
                self.scrollbase.scroll_bottom();
                EventResult::Consumed(None)
            }
            Event::Key(Key::Home) => {
                self.scrollbase.scroll_top();
                EventResult::Consumed(None)
            }
            Event::Key(Key::End) => {
                self.scrollbase.scroll_bottom();
                EventResult::Consumed(None)
            }
            Event::Char('H') => {
                self.scrollbase.scroll_bottom();
                EventResult::Consumed(None)
            }
            Event::Char('/') => {
                // search
                EventResult::Consumed(None)
            }
            _ => EventResult::Ignored,
        }
    }
}

#[derive(Default)]
struct Log {
    s: String,
    lines: Vec<String>,
}

impl Log {
    fn new() -> Self {
        Log {
            s: String::new(),
            lines: vec![],
        }
    }
}

impl vte::Perform for Log {
    fn print(&mut self, c: char) {
        self.s.push(c);
    }

    fn execute(&mut self, _c: u8) {
        let s = self.s.clone();
        if s.is_empty() {
            return;
        }

        self.lines.push(s);
        self.s = String::new();
    }

    fn hook(&mut self, _params: &[i64], _intermediates: &[u8], _ignore: bool, _c: char) {}

    fn put(&mut self, _byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(&mut self, _params: &[i64], _intermediates: &[u8], _ignore: bool, _c: char) {}

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}
