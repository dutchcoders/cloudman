extern crate cursive;

use cursive::theme::{Color, ColorStyle};
use cursive::vec::Vec2;
use cursive::view::View;
use cursive::Printer;
use rusoto_core::Region;

pub struct Column {
    key: String,
    name: String,
}

pub enum BottomBarType {
    Standard,
    Search,
    Filter,
}

pub struct BottomBarView {
    s: String,
    valid: bool,
    profiles: Vec<String>,
    r: Vec<Region>,
    type_: BottomBarType,
}

impl BottomBarView {
    pub fn new(s: &str, r: Vec<Region>) -> Self {
        BottomBarView {
            s: s.to_string(),
            valid: true,
            profiles: ["".to_string()].to_vec(),
            r: r.clone(),
            type_: BottomBarType::Standard,
        }
    }

    pub fn set_type(&mut self, t: BottomBarType) -> &mut Self {
        self.type_ = t;

        self
    }

    pub fn set_profile(&mut self, p: Vec<String>) -> &mut Self {
        self.profiles = p;

        self
    }

    pub fn set_region(&mut self, r: Vec<Region>) -> &mut Self {
        self.r = r;

        self
    }

    pub fn set_valid(&mut self, s: bool) -> &mut Self {
        self.valid = s;

        self
    }

    pub fn set_content(&mut self, s: &str) -> &mut Self {
        self.s = s.to_string();

        self
    }
}

impl View for BottomBarView {
    fn draw(&self, printer: &Printer<'_, '_>) {
        printer.with_color(
            ColorStyle::new(Color::Rgb(145, 198, 194), Color::Rgb(145, 198, 194)),
            |printer| {
                for y in 0..printer.size.y {
                    printer.print_hline((0, y), printer.size.x, " ");
                }
            },
        );

        match self.type_ {
            BottomBarType::Search => {
                let cols = [
                    Column {
                        key: "F3".to_string(),
                        name: "Next  ".to_string(),
                    },
                    Column {
                        key: "Esc".to_string(),
                        name: "Cancel ".to_string(),
                    },
                    Column {
                        key: "  ".to_string(),
                        name: "Search: ".to_string(),
                    },
                ];

                printer.with_color(
                    ColorStyle::new(Color::TerminalDefault, Color::TerminalDefault),
                    |printer| {
                        let mut x = 0;
                        for col in &cols {
                            printer.print((x, 0), &col.key);
                            x += col.key.len();
                            x += col.name.len();
                        }
                    },
                );

                let mut x = 0;

                printer.with_color(
                    ColorStyle::new(Color::Rgb(0, 0, 0), Color::Rgb(145, 198, 194)),
                    |printer| {
                        for col in &cols {
                            x += col.key.len();
                            printer.print((x, 0), &col.name);
                            x += col.name.len();
                        }
                    },
                );

                let cs = if self.valid {
                    ColorStyle::new(Color::Rgb(0, 0, 0), Color::Rgb(145, 198, 194))
                } else {
                    ColorStyle::new(Color::Rgb(255, 0, 0), Color::Rgb(145, 198, 194))
                };

                printer.with_color(cs, |printer| {
                    printer.print((x, 0), &self.s);
                });
            }
            BottomBarType::Filter => {
                let cols = [
                    Column {
                        key: "Enter".to_string(),
                        name: "Done  ".to_string(),
                    },
                    Column {
                        key: "Esc".to_string(),
                        name: "Clear ".to_string(),
                    },
                    Column {
                        key: "  ".to_string(),
                        name: "Filter: ".to_string(),
                    },
                ];

                printer.with_color(
                    ColorStyle::new(Color::TerminalDefault, Color::TerminalDefault),
                    |printer| {
                        let mut x = 0;
                        for col in &cols {
                            printer.print((x, 0), &col.key);
                            x += col.key.len();
                            x += col.name.len();
                        }
                    },
                );

                printer.with_color(
                    ColorStyle::new(Color::Rgb(0, 0, 0), Color::Rgb(145, 198, 194)),
                    |printer| {
                        let mut x = 0;
                        for col in &cols {
                            x += col.key.len();
                            printer.print((x, 0), &col.name);
                            x += col.name.len();
                        }

                        printer.print((x, 0), &self.s);
                    },
                );
            }

            BottomBarType::Standard => {
                let cols = [
                    Column {
                        key: "F1".to_string(),
                        name: "Help  ".to_string(),
                    },
                    Column {
                        key: "F2".to_string(),
                        name: "Connect".to_string(),
                    },
                    Column {
                        key: "F3".to_string(),
                        name: "Search".to_string(),
                    },
                    Column {
                        key: "F4".to_string(),
                        name: "Filter".to_string(),
                    },
                    Column {
                        key: "F5".to_string(),
                        name: "Refresh".to_string(),
                    },
                    Column {
                        key: "F6".to_string(),
                        name: "Actions".to_string(),
                    },
                    Column {
                        key: "F7".to_string(),
                        name: "Region".to_string(),
                    },
                    Column {
                        key: "F10".to_string(),
                        name: "Quit".to_string(),
                    },
                    Column {
                        key: "L".to_string(),
                        name: "Log".to_string(),
                    },
                ];

                printer.with_color(
                    ColorStyle::new(Color::TerminalDefault, Color::TerminalDefault),
                    |printer| {
                        let mut x = 0;
                        for col in &cols {
                            let s = format!("{:<width$}", &col.name, width = 6);
                            printer.print((x, 0), &col.key);
                            x += col.key.len();
                            x += s.len();
                        }
                    },
                );

                printer.with_color(
                    ColorStyle::new(Color::Rgb(0, 0, 0), Color::Rgb(145, 198, 194)),
                    |printer| {
                        let mut x = 0;
                        for col in &cols {
                            let s = format!("{:<width$}", &col.name, width = 6);
                            x += col.key.len();
                            printer.print((x, 0), &s);
                            x += s.len();
                        }
                    },
                );
            }
        }

        printer.with_color(
            ColorStyle::new(Color::Rgb(0, 0, 0), Color::Rgb(145, 198, 194)),
            |printer| {
                let regions: Vec<&str> = self.r.iter().map(|r|r.name()).collect();
                let profiles: Vec<&str> = self.profiles.iter().map(|r|&r[..]).collect();

                let s = format!("{} ({})", regions.join(","), profiles.join(","));
                printer.print((printer.size.x - s.len() - 1, 0), &s);
            },
        );
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        let h = 1;
        let w = &constraint.x;
        //        let h = std::cmp::max(h, &constraint.y);

        Vec2::new(*w, h)
    }
}
