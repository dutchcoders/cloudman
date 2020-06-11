extern crate cursive;

extern crate futures;
extern crate rusoto_core;
extern crate rusoto_ec2;
extern crate tokio_core;
use std::cell::{Cell, RefCell};

extern crate enum_map;

use clap::Clap;

use cursive::theme::{
    BaseColor, BorderStyle, Color, ColorStyle, Effect, PaletteColor, Style, Theme,
};

use cursive::direction::Direction;
use std::env;
use std::panic;

extern crate base64;

use rusoto_core::Region;
use rusoto_core::Region::*;
use std::hash::Hash;
use std::process::Command;
use std::rc::Rc;
use std::str::FromStr;

use rusoto_core::credential::ProfileProvider;
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client, Filter, Instance, Tag};

use cursive::align::HAlign;
use cursive::direction::Orientation;
use cursive::event::*;
use cursive::traits::*;
use cursive::utils::markup::StyledString;
use cursive::view::*;
use cursive::views::*;
use cursive::views::{DebugView, Dialog, EditView, LinearLayout, ResizedView, TextView};
use cursive::CursiveExt;
use rusoto_core::request::HttpClient;

use cursive::event::{Event, EventResult, Key};
use cursive::vec::Vec2;
use cursive::view::View;
use cursive::{Cursive, Printer};

use std::cmp::Ordering;
