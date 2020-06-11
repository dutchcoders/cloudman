extern crate base64;
extern crate cursive;
extern crate enum_map;
extern crate futures;
extern crate rusoto_core;
extern crate rusoto_ec2;
extern crate tokio_core;

use cursive::direction::Direction;
use cursive::event::*;
use cursive::event::{Event, EventResult, Key};
use cursive::theme::{Color, ColorStyle, PaletteColor};
use cursive::vec::Vec2;
use cursive::view::View;
use cursive::view::*;
use cursive::{Cursive, Printer};
use std::cmp::{Eq, Ordering};
use std::hash::Hash;
use std::rc::Rc;

pub type OnSubmit<T> = Option<Rc<dyn Fn(&mut Cursive, Option<T>)>>;

pub struct InstancesView<
    T: TableViewItem<H> + PartialEq,
    H: Eq + Hash + Copy + Clone + Header + 'static,
> {
    instances: Vec<T>,
    scrollbase: ScrollBase,
    current_index: usize,
    columns: Vec<H>,

    on_submit: OnSubmit<T>,
}

pub trait TableViewItem<H>: Clone + Sized
where
    H: Eq + Hash + Copy + Clone + Header + 'static,
{
    /// Method returning a string representation of the item for the
    /// specified column from type `H`.
    fn to_column(&self, column: H) -> String;

    fn to_column_color(&self, column: H) -> ColorStyle;

    /// Method comparing two items via their specified column from type `H`.
    fn cmp(&self, other: &Self, column: H) -> Ordering
    where
        Self: Sized;
}

pub trait Header {
    fn to_header(&self) -> String;
    fn to_header_size(&self, w: usize) -> usize;
}

impl<T: TableViewItem<H> + PartialEq + 'static, H: Eq + Hash + Copy + Clone + Header + 'static>
    Default for InstancesView<T, H>
{
    /// Creates a new empty `TableView` without any columns.
    ///
    /// See [`TableView::new()`].
    fn default() -> Self {
        Self::new()
    }
}

impl<T: TableViewItem<H> + PartialEq + 'static, H: Eq + Hash + Copy + Clone + Header + 'static>
    InstancesView<T, H>
{
    /// Create a new `FlexiLoggerView` which is wrapped in a `ScrollView`.
    pub fn new() -> Self {
        InstancesView {
            instances: vec![],
            scrollbase: ScrollBase::new().right_padding(0),
            current_index: 0,
            columns: vec![],

            on_submit: None,
        }
    }

    pub fn scrollable(instances: &[T]) -> Self {
        InstancesView {
            instances: instances.to_owned(),
            scrollbase: ScrollBase::new().right_padding(0),
            current_index: 0,
            columns: vec![],

            on_submit: None,
        }
    }

    pub fn set_on_submit<F>(&mut self, callback: F)
    where
        F: Fn(&mut Cursive, Option<T>) + 'static,
    {
        self.on_submit = Some(Rc::new(callback));
    }

    pub fn on_submit<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut Cursive, Option<T>) + 'static,
    {
        self.set_on_submit(callback);
        self
    }

    pub fn column(mut self, column: H) -> Self {
        self.columns.push(column);

        self
    }

    pub fn selected_item(&self) -> Option<usize> {
        Some(self.current_index)
    }

    pub fn set_selected_item(&mut self, i: usize) {
        self.current_index = i;
    }

    pub fn item(&self) -> Option<&T> {
        self.instances.get(self.current_index)
    }

    pub fn set_item(&mut self, item: &T) -> &Self {
        match self.instances.iter().position(|t| t.eq(item)) {
            Some(row) => {
                self.set_selected_item(row);
            }
            None => {
                self.set_selected_item(0);
            }
        }

        self
    }

    pub fn items(&self) -> &Vec<T> {
        &self.instances
    }

    pub fn set_instances(&mut self, instances: Vec<T>) -> &Self {
        self.instances = instances;
        self.set_selected_item(0);
        self
    }

    fn make_submit_cb(&self) -> Option<Callback> {
        self.on_submit.clone().map(|cb| {
            Callback::from_fn(move |s| {
                cb(s, None);
            })
        })
    }
}

impl<T: TableViewItem<H> + PartialEq + 'static, H: Eq + Hash + Copy + Clone + Header + 'static> View
    for InstancesView<T, H>
{
    fn draw(&self, printer: &Printer<'_, '_>) {
        self.scrollbase.draw(printer, |printer, i| {
            // draw header
            if i == 0 {
                printer.with_color(
                    ColorStyle::new(Color::Rgb(0, 0, 0), Color::Rgb(185, 202, 74)),
                    |printer| {
                        printer.print_hline((0, 0), printer.size.x, " ");

                        let mut x = 0;

                        for column in self.columns.iter() {
                            let w = column.to_header_size(printer.size.x);

                            let s = format!(
                                "{:.width$} ",
                                format!("{:<width$}", &column.to_header(), width = w),
                                width = w
                            );
                            printer.print((x, 0), &s);

                            x += s.len();
                        }
                    },
                );

                return;
            }

            if let Some(instance) = self.instances.get(i - 1) {
                printer.with_color(
                    if self.current_index == i - 1 {
                        if printer.focused {
                            ColorStyle::new(PaletteColor::HighlightText, PaletteColor::Highlight)
                        } else {
                            ColorStyle::new(
                                PaletteColor::HighlightText,
                                PaletteColor::HighlightInactive,
                            )
                        }
                    } else {
                        ColorStyle::primary()
                    },
                    |p| p.print_hline((0, 0), printer.size.x, " "),
                );

                let mut x = 0;
                for column in self.columns.iter() {
                    let w = column.to_header_size(printer.size.x);
                    let s = format!(
                        "{:.width$} ",
                        format!("{:<width$}", &instance.to_column(*column), width = w),
                        width = w
                    );

                    printer.with_color(
                        if self.current_index == i - 1 {
                            if printer.focused {
                                ColorStyle::new(
                                    PaletteColor::HighlightText,
                                    PaletteColor::Highlight,
                                )
                            } else {
                                ColorStyle::new(
                                    PaletteColor::HighlightText,
                                    PaletteColor::HighlightInactive,
                                )
                            }
                        } else {
                            instance.to_column_color(*column)
                        },
                        |p| p.print((x, 0), &s),
                    );

                    x += s.len();
                }
            }
        });
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        let h = self.instances.len();
        let h = std::cmp::max(h, constraint.y);

        self.scrollbase.set_heights(constraint.y, h);

        constraint
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        true
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Key(Key::Up) => {
                if self.current_index > 0 {
                    self.current_index -= 1;
                }
                EventResult::Consumed(None)
            }
            Event::Key(Key::Down) => {
                if self.current_index + 1 < self.instances.len() {
                    self.current_index += 1;
                }
                EventResult::Consumed(None)
            }
            Event::Char('g') => {
                self.scrollbase.scroll_top();
                EventResult::Consumed(None)
            }
            Event::Key(Key::PageUp) => {
                if self.current_index < 10 {
                    self.current_index = 0;
                } else {
                    self.current_index -= 10;
                }
                EventResult::Consumed(None)
            }
            Event::Key(Key::PageDown) => {
                let idx = std::cmp::min(self.instances.len() - 1, self.current_index + 10);
                self.current_index = idx;
                EventResult::Consumed(None)
            }
            Event::Key(Key::Home) => {
                self.current_index = 0;
                EventResult::Consumed(None)
            }
            Event::Key(Key::End) => {
                self.current_index = self.instances.len() - 1;
                EventResult::Consumed(None)
            }
            Event::Shift(Key::Home) => {
                self.current_index = 0;
                EventResult::Consumed(None)
            }
            Event::Shift(Key::End) => {
                self.current_index = self.instances.len() - 1;
                EventResult::Consumed(None)
            }
            Event::Key(Key::Enter) => EventResult::Consumed(self.make_submit_cb()),
            Event::Char('H') => {
                self.current_index = self.instances.len() - 1;
                EventResult::Consumed(None)
            }
            _ => EventResult::Ignored,
        }
    }
}
