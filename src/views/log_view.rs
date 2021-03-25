extern crate cursive;

use cursive::{Rect, direction::Direction, view::scroll::{self, Core}};
use cursive::event::{Event, EventResult, Key};
use cursive::vec::Vec2;
use cursive::{Printer, view::View};

cursive::impl_scroller!(LogView::core);

pub struct LogView {
    lines: Vec<String>,

    core: cursive::view::scroll::Core,
}

impl LogView {
    pub fn scrollable(buf: &[u8]) -> Self {
        let lines = parse_lines(&buf);

        LogView {
            lines,
            core: Core::new(),
        }
    }

    fn inner_required_size(&mut self, _req: Vec2) -> Vec2 {
        Vec2::new(80, self.lines.len())
    }

    fn inner_on_event(&mut self, event: Event) -> EventResult {
        let height = self.core.content_viewport().height();
        match event {
            Event::Key(Key::Up) => {
                self.core.scroll_up(1);
                EventResult::Consumed(None)
            }
            Event::Key(Key::Down) => {
                self.core.scroll_down(1);
                EventResult::Consumed(None)
            }
            Event::Key(Key::PageUp) => {
                self.core.scroll_up(height);
                EventResult::Consumed(None)
            }
            Event::Key(Key::PageDown) => {
                self.core.scroll_down(height);
                EventResult::Consumed(None)
            }
            Event::Char('g') => {
                self.core.scroll_to_top();
                EventResult::Consumed(None)
            }
            Event::Shift(Key::Home) => {
                self.core.scroll_to_top();
                EventResult::Consumed(None)
            }
            Event::Shift(Key::End) => {
                self.core.scroll_to_bottom();
                EventResult::Consumed(None)
            }
            Event::Key(Key::Home) => {
                self.core.scroll_to_top();
                EventResult::Consumed(None)
            }
            Event::Key(Key::End) => {
                self.core.scroll_to_bottom();
                EventResult::Consumed(None)
            }
            Event::Char('H') => {
                self.core.scroll_to_bottom();
                EventResult::Consumed(None)
            }
            Event::Char('/') => {
                // search
                EventResult::Consumed(None)
            }
            _ => EventResult::Ignored
        }
    }

    fn inner_important_area(&self, size: Vec2) -> Rect {
        Rect::from_size((0, 0), (size.x, self.lines.len()))
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
        let lines = self.lines.clone();
        scroll::draw_lines(self, &printer, |_, printer, i| {
            // ignore the first line, as it is incomplete
            if let Some(line) = lines.get(i + 1) {
                printer.print((0, 0), line);
            } else {
                printer.print((0, 0), "⍇");
            }

        });
    }

    fn required_size(&mut self, req: Vec2) -> Vec2 {
        scroll::required_size(
            self,
            req,
            true,
            Self::inner_required_size,
        )
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        true
    }

    fn layout(&mut self, size: Vec2) {
        scroll::layout(
            self,
            size,
            true,
            |_s, _size| (),
            Self::inner_required_size,
        );
    }

    fn important_area(&self, size: Vec2) -> Rect {
        scroll::important_area(
            self,
            size,
            Self::inner_important_area,
        )
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        scroll::on_event(
            self,
            event,
            Self::inner_on_event,
            Self::inner_important_area,
        )
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
