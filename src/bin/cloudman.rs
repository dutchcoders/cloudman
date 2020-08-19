extern crate cursive;
extern crate dirs;
extern crate rusoto_core;
extern crate rusoto_ec2;
extern crate tokio;

use clap::Clap;
use cursive::align::HAlign;
use cursive::direction::Orientation;
use cursive::event::{Event, EventResult, Key};
use cursive::theme::{BaseColor, Color, ColorStyle, PaletteColor};
use cursive::traits::*;
use cursive::view::View;
use cursive::view::*;
use cursive::views::{
    Canvas, Dialog, EditView, LinearLayout, OnEventView, ResizedView, SelectView, TextContent,
    TextView,
};
use cursive::Cursive;
use cursive::CursiveExt;
use rusoto_core::credential::{ProfileProvider,EnvironmentProvider};
use rusoto_core::request::HttpClient;
use rusoto_core::Region;
use rusoto_core::Region::*;
use rusoto_ec2::{
    DescribeInstancesRequest, Ec2, Ec2Client, RebootInstancesRequest, StartInstancesRequest,
    TerminateInstancesRequest,
    StopInstancesRequest, Tag,
};
use std::cmp::Ordering;
use std::env;
use std::error::Error;
use std::hash::Hash;
use std::panic;
use std::process::Command;
use std::str::FromStr;

use cloudman_rs::views::{
    BottomBarType, BottomBarView, Foo, Header, InstancesView, KeyCodeView, LogView, TableViewItem,
};

// Use of a mod or pub mod is not actually necessary.
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BasicColumn {
    InstanceID,
    Name,
    Architecture,
    Region,
    Profile,
    VpcID,
    Type,
    Key,
    State,
    PublicIp,
    PrivateIp,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum Actions {
    Start,
    Stop,
    Terminate,
    Reboot,
}

#[derive(Clone, PartialEq)]
pub struct Instance {
    instance: rusoto_ec2::Instance,
    profile: String,
    region: Region,
}
fn find_tag(key: String, tags: Option<Vec<Tag>>) -> Option<String> {
    match tags {
        Some(tags) => {
            match tags
                .iter()
                .find(|t| t.key.clone().unwrap().eq_ignore_ascii_case(&key))
            {
                Some(tag) => tag.clone().value,
                None => None,
            }
        }
        None => None,
    }
}

impl TableViewItem<BasicColumn> for Instance {
    fn to_column_color(&self, column: BasicColumn) -> ColorStyle {
        match column {
            BasicColumn::Name => {
                ColorStyle::new(Color::Dark(BaseColor::Green), Color::TerminalDefault)
            }
            BasicColumn::State => match self.instance.state.as_ref().unwrap().code {
                Some(16) => ColorStyle::new(Color::Light(BaseColor::Green), Color::TerminalDefault),
                _ => ColorStyle::new(Color::Light(BaseColor::Red), Color::TerminalDefault),
            },
            _ => ColorStyle::primary(),
        }
    }

    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::InstanceID => self
                .instance
                .instance_id
                .clone()
                .unwrap_or_else(|| "".to_string()),
            BasicColumn::Name => find_tag("name".to_string(), self.instance.tags.clone())
                .unwrap_or_else(|| "".to_string()),
            BasicColumn::Region => self.region.clone().name().to_string(),
            BasicColumn::Profile => self.profile.to_string(),
            BasicColumn::Architecture => self
                .instance
                .architecture
                .clone()
                .unwrap_or_else(|| "".to_string()),
            BasicColumn::VpcID => self
                .instance
                .vpc_id
                .clone()
                .unwrap_or_else(|| "".to_string()),
            BasicColumn::Type => self
                .instance
                .instance_type
                .clone()
                .unwrap_or_else(|| "".to_string()),
            BasicColumn::Key => self
                .instance
                .key_name
                .clone()
                .unwrap_or_else(|| "".to_string()),
            BasicColumn::State => self
                .instance
                .state
                .clone()
                .unwrap()
                .name
                .unwrap_or_else(|| "".to_string()),
            BasicColumn::PublicIp => self
                .instance
                .public_ip_address
                .clone()
                .unwrap_or_else(|| "".to_string()),
            BasicColumn::PrivateIp => self
                .instance
                .private_ip_address
                .clone()
                .unwrap_or_else(|| "".to_string()),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            BasicColumn::Name => self.instance.instance_id.cmp(&other.instance.instance_id),
            BasicColumn::InstanceID => self.instance.instance_id.cmp(&other.instance.instance_id),
            BasicColumn::Region => self.region.name().cmp(&other.region.name()),
            BasicColumn::Profile => self.profile.cmp(&other.profile),
            BasicColumn::Architecture => {
                self.instance.architecture.cmp(&other.instance.architecture)
            }
            BasicColumn::VpcID => self.instance.vpc_id.cmp(&other.instance.vpc_id),
            BasicColumn::Type => self
                .instance
                .instance_type
                .cmp(&other.instance.instance_type),
            BasicColumn::Key => self.instance.key_name.cmp(&other.instance.key_name),
            BasicColumn::State => self.instance.key_name.cmp(&other.instance.key_name),
            BasicColumn::PublicIp => self
                .instance
                .public_ip_address
                .cmp(&other.instance.public_ip_address),
            BasicColumn::PrivateIp => self
                .instance
                .private_ip_address
                .cmp(&other.instance.private_ip_address),
        }
    }
}

// TODO: add parallel loading of multiple envs
fn get_instances_with_region(
    profiles: Vec<String>,
    regions: Vec<Region>,
) -> Result<Vec<Instance>, Box<dyn Error>> {
    let mut runtime = tokio::runtime::Runtime::new().unwrap();

    let mut instances: Vec<Instance> = vec![];
    for profile in profiles.iter() {
        for region in regions.iter() {
            let client = new_ec2client(region, profile)?;

            let req: DescribeInstancesRequest = ec2_describe_input();

            let ft = client.describe_instances(req);

            let response = runtime.block_on(ft)?;
            if let Some(reservations) = response.reservations {
                for reservation in reservations {
                    if let Some(res_instances) = reservation.instances {
                        for instance in res_instances {
                            instances.push(Instance {
                                instance: instance,
                                region: region.clone(),
                                profile: profile.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(instances)
}

#[derive(Clap)]
#[clap(version = built_info::PKG_VERSION, author = built_info::PKG_AUTHORS)]
struct Opts {
    #[clap(short, long, about("One or more regions to use"))]
    region: Vec<String>,

    #[clap(short, long, about("One or more profiles to use"))]
    profile: Vec<String>,

    #[clap(long, about("Disable dry run"))]
    disable_dry_run: bool,

    #[clap(long, about("Usen environment credentials"))]
    use_env: bool,
}

fn main() {
    match panic::catch_unwind(|| {
        run();
    }) {
        Ok(_) => {} // we're ok
        Err(_) => run_bsod(),
    }
}

impl Header for BasicColumn {
    fn to_header_size(&self, w: usize) -> usize {
        match self {
            BasicColumn::InstanceID => 19,
            BasicColumn::Name => (40 * w) / 160,
            BasicColumn::Architecture => 8,
            BasicColumn::VpcID => 22,
            BasicColumn::Profile => 12,
            BasicColumn::Region => 12,
            BasicColumn::Type => 12,
            BasicColumn::Key => (20 * w) / 160,
            BasicColumn::State => 10,
            BasicColumn::PublicIp => 15,
            BasicColumn::PrivateIp => 15,
        }
    }

    fn to_header(&self) -> String {
        match self {
            BasicColumn::InstanceID => "instance-id".to_string(),
            BasicColumn::Name => "name".to_string(),
            BasicColumn::Architecture => "arch".to_string(),
            BasicColumn::VpcID => "vpc-id".to_string(),
            BasicColumn::Profile => "profile".to_string(),
            BasicColumn::Region => "region".to_string(),
            BasicColumn::Type => "type".to_string(),
            BasicColumn::Key => "key".to_string(),
            BasicColumn::State => "state".to_string(),
            BasicColumn::PublicIp => "public-ip".to_string(),
            BasicColumn::PrivateIp => "private-ip".to_string(),
        }
    }
}

fn cursive_new() -> Cursive {
    let mut siv = Cursive::default();
    siv.set_theme(cursive::theme::load_toml(include_str!("../ncurses_theme.toml")).unwrap());

    siv
}

fn run() {
    let opts: Opts = Opts::parse();

    cursive::logger::init();

    let regions: Vec<Region> = {
        if opts.region.is_empty() {
            let regions: Vec<Region> = [Region::default()].to_vec();
            regions
        } else {
            opts.region
                .iter()
                .flat_map(|r| r.split(","))
                .map(|r| Region::from_str(&r).unwrap())
                .collect()
        }
    };

    let profiles: Vec<String> = {
        if opts.profile.is_empty() {
            let profiles: Vec<String> = ["default".to_string()].to_vec();
            profiles
        } else {
            opts.profile
                .iter()
                .flat_map(|r| r.split(","))
                .map(|r| r.to_string())
                .collect()
        }
    };

    let instances = match get_instances_with_region(profiles.clone(), regions.clone()) {
        Ok(instances) => instances,
        Err(err) => {
            eprintln!("Could not retrieve instances\n\n{}", err);
            std::process::exit(1);
        }
    };

    let mut siv = cursive_new();

    let mut rv = ReturnValues::default();
    rv.profiles = profiles;
    rv.instances = instances.clone();
    rv.regions = regions.clone();
    rv.dry_run = !opts.disable_dry_run;

    siv.set_user_data::<ReturnValues>(rv);

    let mut layout = LinearLayout::new(Orientation::Vertical);

    let dialog_title = TextView::new(built_info::PKG_NAME)
        .h_align(HAlign::Center)
        .with_name("title");

    layout.add_child(dialog_title);

    let mut iv = InstancesView::<Instance, BasicColumn>::scrollable(&instances)
        .column(BasicColumn::InstanceID)
        .column(BasicColumn::Name)
        .column(BasicColumn::Architecture)
        .column(BasicColumn::Region)
        .column(BasicColumn::Profile)
        .column(BasicColumn::VpcID)
        .column(BasicColumn::Type)
        .column(BasicColumn::Key)
        .column(BasicColumn::State)
        .column(BasicColumn::PublicIp)
        .column(BasicColumn::PrivateIp);

    iv.set_on_submit(|s: &mut Cursive, _instance: Option<Instance>| {
        let table = s
            .find_name::<InstancesView<Instance, BasicColumn>>("instances")
            .unwrap();

        if let Some(instance) = table.item() {
            instance_details(s, instance);
        }
    });

    layout.add_child(iv.with_name("instances"));

    let bottom_bar = BottomBarView::new(&"".to_string(), regions.clone()).with_name("bottom_bar");

    layout.add_child(bottom_bar);

    siv.add_fullscreen_layer(
        OnEventView::new(layout)
            .on_event('/', on_search)
            .on_event(Key::F3, on_search)
            .on_event(Key::F4, on_filter)
            .on_event(Event::CtrlChar('k'), |s| {
                let d = KeyCodeView::new(10).full_width().fixed_height(10);
                s.add_layer(d);
            })
            .on_event('l', |s| {
                let table = s
                    .find_name::<InstancesView<Instance, BasicColumn>>("instances")
                    .unwrap();

                if let Some(instance) = table.item() {
                    instance_log(s, instance);
                }
            })
            .on_event(Key::Esc, |s| reset_filter(s))
            .on_event(Key::F9, |s| change_profile(s))
            .on_event(Key::F7, |s| change_region(s))
            .on_event(Key::F6, |s| action(s))
            .on_event(Key::F5, |s| refresh(s))
            .on_event(Key::F1, |s| help(s))
            .on_event(Key::F2, |s| {
                let table = s
                    .find_name::<InstancesView<Instance, BasicColumn>>("instances")
                    .unwrap();

                if let Some(instance) = table.item() {
                    if connect(instance).is_err() {
                        let d = Dialog::around(TextView::new("Not running within tmux."))
                            .title("Error")
                            .button("Cancel", |s| {
                                s.pop_layer();
                            });

                        let dl = event_view(d);

                        s.add_layer(dl);
                    }
                }
            })
            .on_event('q', |s| s.quit()),
    );

    update_bottom_bar(&mut siv);

    siv.add_global_callback(Key::F10, |s| s.quit());

    siv.add_global_callback('s', |s| s.toggle_debug_console());

    siv.run()
}

struct ReturnValues {
    profiles: Vec<String>,
    regions: Vec<Region>,
    search: String,
    search_found: bool,
    searching: bool,
    filter: String,
    filtering: bool,
    instances: Vec<Instance>,
    dry_run: bool,
}

impl Default for ReturnValues {
    fn default() -> Self {
        Self {
            profiles: vec!["default".to_string()],
            regions: vec![Region::default()],
            search: "".to_string(),
            searching: false,
            search_found: false,
            filter: "".to_string(),
            filtering: false,
            instances: vec![],
            dry_run: false,
        }
    }
}

fn on_filter(s: &mut Cursive) {
    s.with_user_data(|v: &mut ReturnValues| {
        v.filtering = true;
    });

    let ud = s.user_data::<ReturnValues>().unwrap();

    let mut overlay = Foo::with_string(&ud.filter.to_string());

    overlay.set_on_search(|s, ss, _| {
        s.with_user_data(|v: &mut ReturnValues| {
            v.filter = ss.to_string();
        });

        let ud = s.user_data::<ReturnValues>().unwrap();

        let filtered_instances: Vec<Instance> = ud
            .instances
            .clone()
            .into_iter()
            .filter(|i| {
                find_tag("Name".to_string(), i.instance.tags.clone())
                    .unwrap_or_default()
                    .contains(ss)
                    || i.region.name().contains(ss)
                    || i.profile.contains(ss)
            })
            .collect();

        let mut table = s
            .find_name::<InstancesView<Instance, BasicColumn>>("instances")
            .unwrap();

        let item = table.item();

        match item {
            Some(item) => {
                let item2 = item.clone();
                table.set_instances(filtered_instances);
                table.set_item(&item2);
            }
            None => {
                table.set_instances(filtered_instances);
            }
        }

        update_bottom_bar(s);
    });

    overlay.set_on_search_next(|_, _, _| {});

    overlay.set_on_cancel(|s| {
        reset_filter(s);
        s.pop_layer();
    });

    overlay.set_on_close(|s| {
        s.with_user_data(|v: &mut ReturnValues| {
            v.filtering = false;
        });

        update_bottom_bar(s);
        s.pop_layer();
    });

    s.add_fullscreen_layer(overlay);
    update_bottom_bar(s);
}

fn reset_filter(s: &mut Cursive) {
    s.with_user_data(|v: &mut ReturnValues| {
        v.filtering = false;
        v.filter = "".to_string();
    });

    let mut table = s
        .find_name::<InstancesView<Instance, BasicColumn>>("instances")
        .unwrap();

    let ud = s.user_data::<ReturnValues>().unwrap();

    let instances = ud.instances.clone();

    let item = table.item();

    match item {
        Some(item) => {
            let item2 = item.clone();
            table.set_instances(instances);
            table.set_item(&item2);
        }
        None => {
            table.set_instances(instances);
        }
    }

    update_bottom_bar(s);
}

fn on_search(s: &mut Cursive) {
    s.with_user_data(|v: &mut ReturnValues| {
        v.search = String::new();
        v.searching = true;
    });

    let mut overlay = Foo::default();
    overlay.set_on_search(|s, ss, _| {
        let mut table = s
            .find_name::<InstancesView<Instance, BasicColumn>>("instances")
            .unwrap();

        let instances = table.items();

        match instances.iter().position(|i| {
            find_tag("Name".to_string(), i.instance.tags.clone())
                .unwrap_or_default()
                .to_lowercase()
                .contains(&ss.to_lowercase())
        }) {
            Some(idx) => {
                table.set_selected_item(idx);
                s.with_user_data(|v: &mut ReturnValues| {
                    v.search_found = true;
                    v.search = ss.to_string();
                });
            }
            None => {
                s.with_user_data(|v: &mut ReturnValues| {
                    v.search_found = false;
                    v.search = ss.to_string();
                });
            }
        }

        update_bottom_bar(s);
    });

    overlay.set_on_search_next(|s, ss, _| {
        let mut table = s
            .find_name::<InstancesView<Instance, BasicColumn>>("instances")
            .unwrap();

        let instances = table.items();

        let selected_row = table.selected_item().unwrap();

        if let Some(idx) = instances.iter().skip(selected_row + 1).position(|i| {
            find_tag("Name".to_string(), i.instance.tags.clone())
                .unwrap_or_default()
                .to_lowercase()
                .contains(&ss.to_lowercase())
        }) {
            table.set_selected_item(idx + selected_row + 1);
        } else if let Some(idx) = instances.iter().position(|i| {
            find_tag("Name".to_string(), i.instance.tags.clone())
                .unwrap_or_default()
                .to_lowercase()
                .contains(&ss.to_lowercase())
        }) {
            table.set_selected_item(idx);
        } else {
        }
    });

    overlay.set_on_cancel(|s| {
        s.with_user_data(|v: &mut ReturnValues| {
            v.searching = false;
        });

        update_bottom_bar(s);
        s.pop_layer();
    });

    overlay.set_on_close(|s| {
        s.with_user_data(|v: &mut ReturnValues| {
            v.searching = false;
        });

        update_bottom_bar(s);
        s.pop_layer();
    });

    s.add_fullscreen_layer(overlay);
    update_bottom_bar(s);
}

fn update_bottom_bar(s: &mut Cursive) {
    let mut bottom_bar = s.find_name::<BottomBarView>("bottom_bar").unwrap();

    let ud = s.user_data::<ReturnValues>().unwrap();

    if ud.searching {
        bottom_bar
            .set_content(&ud.search.clone())
            .set_valid(ud.search_found)
            .set_region(ud.regions.clone())
            .set_profile(ud.profiles.clone())
            .set_type(BottomBarType::Search);
    } else if ud.filtering {
        bottom_bar
            .set_content(&ud.filter.clone())
            .set_region(ud.regions.clone())
            .set_profile(ud.profiles.clone())
            .set_type(BottomBarType::Filter);
    } else {
        bottom_bar
            .set_region(ud.regions.clone())
            .set_profile(ud.profiles.clone())
            .set_type(BottomBarType::Standard);
    }
}

fn connect(instance: &Instance) -> Result<(), Box<dyn Error>> {
    env::var("TMUX")?;

    Command::new("tmux")
        .arg("split-window")
        .arg("-h")
        .arg("bash")
        .arg("-c")
        .arg(format!(r#"aws ssm start-session --profile "{:?}" --region "{:?}" --target "{:}"; read -n 1 -s -r -p "Press any key to continue""#, &instance.profile, instance.region.name(), instance.instance.instance_id.clone().unwrap()))
        .output()?;

    Ok(())
}

fn refresh(s: &mut Cursive) {
    let mut iv = s
        .find_name::<InstancesView<Instance, BasicColumn>>("instances")
        .unwrap();

    let ud = s.user_data::<ReturnValues>().unwrap();

    let item = iv.item();

    match get_instances_with_region(ud.profiles.clone(), ud.regions.clone()) {
        Ok(instances) => {
            let instances: Vec<Instance> = if ud.filter != "" {
                instances
                    .into_iter()
                    .filter(|i| {
                        find_tag("name".to_string(), i.instance.tags.clone())
                            .unwrap_or_else(|| "".to_string())
                            .contains(&ud.filter)
                    })
                    .collect()
            } else {
                instances
            };


            match item {
                Some(item) => {
                    let item2 = item.clone();
                    iv.set_instances(instances);
                    iv.set_item(&item2);
                }
                None => {
                    iv.set_instances(instances);
                }
            }
        }
        Err(err) => {
            let d = Dialog::around(TextView::new(format!(
                "Could not retrieve instances.\n\n{}",
                err
            )))
            .title("Error")
            .button("Cancel", |s| {
                s.pop_layer();
            });

            let dl = event_view(d);

            s.add_layer(dl);
        }
    }
}

fn new_ec2client(
    region: &rusoto_core::Region,
    profile: &str,
) -> Result<rusoto_ec2::Ec2Client, rusoto_core::request::TlsError> {
    let opts: Opts = Opts::parse();

    let http_client = HttpClient::new()?;

    if opts.use_env {
        let provider = EnvironmentProvider::default();
        Ok(Ec2Client::new_with(http_client, provider, region.clone()))
    } else {
        let aws_creds_dir: String = dirs::home_dir().unwrap().to_str().unwrap().to_owned() + "/.aws/credentials";
        let provider = ProfileProvider::with_configuration(aws_creds_dir, profile);
        Ok(Ec2Client::new_with(http_client, provider, region.clone()))
    }
}

fn get_instance_log(instance: &Instance) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let client = new_ec2client(&instance.region, &instance.profile)?;

    let req = rusoto_ec2::GetConsoleOutputRequest {
        instance_id: instance.instance.instance_id.clone().unwrap(),
        ..Default::default()
    };

    let mut runtime = tokio::runtime::Runtime::new().unwrap();

    let ft = client.get_console_output(req);

    let response = runtime.block_on(ft)?;

    let output = response.output.unwrap();

    let buf = &base64::decode(&output).unwrap()[..];

    Ok(buf.to_vec())
}

fn instance_log(siv: &mut Cursive, instance: &Instance) {
    match get_instance_log(&instance) {
        Ok(buf) => {
            let mut dl = LinearLayout::new(Orientation::Vertical);

            let instance = instance.clone();

            let dialog_title = TextView::new(format!(
                "{} ({:})",
                built_info::PKG_NAME,
                instance.instance.instance_id.unwrap()
            ))
            .h_align(HAlign::Center)
            .with_name("title");

            dl.add_child(dialog_title);

            dl.add_child(ResizedView::new(
                SizeConstraint::Full,
                SizeConstraint::Full,
                LogView::scrollable(&buf),
            ));

            let dl = event_view(dl);

            siv.add_fullscreen_layer(dl);
        }
        Err(err) => {
            let d = Dialog::around(TextView::new(format!(
                "Could not retrieve the instance log.\n\n{}",
                err
            )))
            .title("Error")
            .button("Cancel", |s| {
                s.pop_layer();
            });

            let dl = event_view(d);

            siv.add_layer(dl);
        }
    }
}

fn event_view<V: View + 'static>(v: V) -> OnEventView<V> {
    OnEventView::new(v)
        .on_event(Key::Esc, |s| {
            s.pop_layer();
        })
        .on_event('q', |s| {
            s.pop_layer();
        })
}

fn instance_details(siv: &mut Cursive, instance: &Instance) {
    let mut dl = LinearLayout::new(Orientation::Vertical);

    let instance = instance.clone();

    let dialog_title = TextView::new(format!(
        "{} ({:})",
        built_info::PKG_NAME,
        instance.instance.instance_id.clone().unwrap()
    ))
    .h_align(HAlign::Center)
    .with_name("title");

    dl.add_child(dialog_title);

    let canvas = Canvas::new(instance).with_draw(|instance, printer| {
        let x = [
            (
                "instance-id",
                &instance.instance.instance_id.clone().unwrap(),
            ),
            (
                "name",
                &find_tag("name".to_string(), instance.instance.tags.clone())
                    .unwrap_or_else(|| "".to_string()),
            ),
            (
                "architecture",
                &instance
                    .instance
                    .architecture
                    .clone()
                    .unwrap_or_else(|| "".to_string()),
            ),
            (
                "vpc-id",
                &instance
                    .instance
                    .vpc_id
                    .clone()
                    .unwrap_or_else(|| "".to_string()),
            ),
            (
                "subnet_type",
                &instance
                    .instance
                    .subnet_id
                    .clone()
                    .unwrap_or_else(|| "".to_string()),
            ),
            (
                "instance_type",
                &instance
                    .instance
                    .instance_type
                    .clone()
                    .unwrap_or_else(|| "".to_string()),
            ),
            (
                "key_name",
                &instance
                    .instance
                    .key_name
                    .clone()
                    .unwrap_or_else(|| "".to_string()),
            ),
            (
                "state",
                &instance.instance.state.clone().unwrap().name.unwrap(),
            ),
            (
                "public ip",
                &instance
                    .instance
                    .public_ip_address
                    .clone()
                    .unwrap_or_else(|| "".to_string()),
            ),
            (
                "public dns",
                &instance
                    .instance
                    .public_dns_name
                    .clone()
                    .unwrap_or_else(|| "".to_string()),
            ),
            (
                "private ip",
                &instance
                    .instance
                    .private_ip_address
                    .clone()
                    .unwrap_or_else(|| "".to_string()),
            ),
            (
                "private dns",
                &instance
                    .instance
                    .private_dns_name
                    .clone()
                    .unwrap_or_else(|| "".to_string()),
            ),
            (
                "placement",
                &instance
                    .instance
                    .placement
                    .clone()
                    .unwrap()
                    .group_name
                    .unwrap(),
            ),
            (
                "lifecycle",
                &instance
                    .instance
                    .instance_lifecycle
                    .clone()
                    .unwrap_or_else(|| "".to_string()),
            ),
            (
                "image-id",
                &instance
                    .instance
                    .image_id
                    .clone()
                    .unwrap_or_else(|| "".to_string()),
            ),
            (
                "ramdisk-id",
                &instance
                    .instance
                    .ramdisk_id
                    .clone()
                    .unwrap_or_else(|| "".to_string()),
            ),
            (
                "root device",
                &format!(
                    "{:} ({:})",
                    &instance
                        .instance
                        .root_device_name
                        .clone()
                        .unwrap_or_else(|| "".to_string()),
                    &instance
                        .instance
                        .root_device_type
                        .clone()
                        .unwrap_or_else(|| "".to_string()),
                ),
            ),
            (
                "state",
                &instance.instance.state.clone().unwrap().name.unwrap(),
            ),
            //("state-reason", &instance.state_reason.clone().unwrap().message.unwrap()),
        ];

        for (i, pair) in x.iter().enumerate() {
            printer.with_color(
                ColorStyle::new(PaletteColor::TitleSecondary, PaletteColor::Background),
                |p| p.print((0, i + 1), &format!("{:>20}:", pair.0)),
            );
            printer.print((22, i + 1), &pair.1.to_string());
        }

        let mut y = 22;

        printer.with_color(
            ColorStyle::new(PaletteColor::TitleSecondary, PaletteColor::Background),
            |p| p.print((0, y), "security groups"),
        );
        y += 1;

        let security_groups = instance.instance.security_groups.clone();

        for sg in security_groups.unwrap().iter() {
            printer.print(
                (0, y),
                &format!(
                    "{:>20}: {:}",
                    &sg.group_id.clone().unwrap(),
                    &sg.group_name.clone().unwrap(),
                ),
            );

            y += 1;
        }

        y += 1;

        printer.with_color(
            ColorStyle::new(PaletteColor::TitleSecondary, PaletteColor::Background),
            |p| p.print((0, y), "network interfaces"),
        );
        y += 1;

        let network_interfaces = instance.instance.network_interfaces.clone();
        for sg in network_interfaces.unwrap().iter() {
            printer.print(
                (0, y),
                &format!(
                    "{:>20}: {:}",
                    &sg.network_interface_id.clone().unwrap(),
                    &sg.description.clone().unwrap(),
                ),
            );

            y += 1;
        }
    });

    dl.add_child(ResizedView::new(
        SizeConstraint::Full,
        SizeConstraint::Full,
        canvas,
    ));

    let dl = event_view(dl);

    siv.add_fullscreen_layer(dl);
}

fn help(siv: &mut Cursive) {
    let mut dl = LinearLayout::new(Orientation::Vertical);

    let content = TextContent::new(include_str!("../help.md"));
    let view = TextView::new_with_content(content);

    dl.add_child(ResizedView::new(
        SizeConstraint::Full,
        SizeConstraint::Full,
        view,
    ));

    let dl = event_view(dl);

    siv.add_fullscreen_layer(dl);
}

fn run_bsod() {
    let mut siv = cursive_new();

    let d = Dialog::around(TextView::new(
        "Cloudman has encountered an error and needs to exit.",
    ))
    .title("Panic")
    .button("Exit", |s| s.quit());

    siv.add_layer(d);
    siv.run();
}

fn action(s: &mut Cursive) {
    let mut select = SelectView::<Actions>::new()
        .h_align(HAlign::Center)
        .autojump()
        .item("start", Actions::Start)
        .item("stop", Actions::Stop)
        .item("terminate", Actions::Terminate)
        .item("reboot", Actions::Reboot);

    fn ok(s: &mut Cursive, action: &Actions) {
        let table = &s
            .find_name::<InstancesView<Instance, BasicColumn>>("instances")
            .unwrap();

        let instance = table.item();

        if instance.is_none() {
            return;
        }

        let ud = s.user_data::<ReturnValues>().unwrap();

        let client = new_ec2client(&instance.unwrap().region, &instance.unwrap().profile);
        if client.is_err() {
            return;
        };

        let client = client.unwrap();
        match action {
            Actions::Start => {
                let req = StartInstancesRequest {
                    dry_run: Some(ud.dry_run),
                    instance_ids: [instance.unwrap().instance.instance_id.clone().unwrap()]
                        .to_vec(),
                    ..Default::default()
                };

                let ft = client.start_instances(req);

                let mut runtime = tokio::runtime::Runtime::new().unwrap();

                match runtime.block_on(ft) {
                    Ok(_) => {
                        match s.cb_sink().send(Box::new(|s| {
                            let d = Dialog::around(TextView::new("The instance will start."))
                                .title("Start instance")
                                .button("Cancel", |s| {
                                    s.pop_layer();
                                });

                            let dl = event_view(d);

                            s.add_layer(dl);
                        })) {
                            Ok(_) => (),
                            Err(err) => {
                                s.pop_layer();
                                error_dialog(s, "Could not start instance.", &format!("{}", err));
                            }
                        };
                    }
                    Err(err) => {
                        s.pop_layer();
                        error_dialog(s, "Could not start instance.", &format!("{}", err));
                    }
                };
            }
            Actions::Stop => {
                let req = StopInstancesRequest {
                    dry_run: Some(ud.dry_run),
                    instance_ids: [instance.unwrap().instance.instance_id.clone().unwrap()]
                        .to_vec(),
                    ..Default::default()
                };

                let ft = client.stop_instances(req);

                let mut runtime = tokio::runtime::Runtime::new().unwrap();

                match runtime.block_on(ft) {
                    Ok(_) => {
                        match s.cb_sink().send(Box::new(|s| {
                            let d = Dialog::around(TextView::new("The instance will be stopped."))
                                .title("Stop instance")
                                .button("Cancel", |s| {
                                    s.pop_layer();
                                });

                            let dl = event_view(d);

                            s.add_layer(dl);
                        })) {
                            Ok(_) => (),
                            Err(err) => {
                                s.pop_layer();
                                error_dialog(s, "Could not stop instance.", &format!("{}", err));
                            }
                        };
                    }
                    Err(err) => {
                        s.pop_layer();
                        error_dialog(s, "Could not stop instance.", &format!("{}", err));
                    }
                };
            }
            Actions::Terminate => {
                let req = TerminateInstancesRequest {
                    dry_run: Some(ud.dry_run),
                    instance_ids: [instance.unwrap().instance.instance_id.clone().unwrap()]
                        .to_vec(),
                    ..Default::default()
                };

                let ft = client.terminate_instances(req);

                let mut runtime = tokio::runtime::Runtime::new().unwrap();

                match runtime.block_on(ft) {
                    Ok(_) => {
                        match s.cb_sink().send(Box::new(|s| {
                            let d = Dialog::around(TextView::new("The instance will be terminated."))
                                .title("Stop instance")
                                .button("Cancel", |s| {
                                    s.pop_layer();
                                });

                            let dl = event_view(d);

                            s.add_layer(dl);
                        })) {
                            Ok(_) => (),
                            Err(err) => {
                                s.pop_layer();
                                error_dialog(s, "Could not terminate instance.", &format!("{}", err));
                            }
                        };
                    }
                    Err(err) => {
                        s.pop_layer();
                        error_dialog(s, "Could not terminate instance.", &format!("{}", err));
                    }
                };
            }
            Actions::Reboot => {
                let req = RebootInstancesRequest {
                    dry_run: Some(ud.dry_run),
                    instance_ids: [instance.unwrap().instance.instance_id.clone().unwrap()]
                        .to_vec(),
                    //..Default::default()
                };

                let ft = client.reboot_instances(req);

                let mut runtime = tokio::runtime::Runtime::new().unwrap();

                match runtime.block_on(ft) {
                    Ok(_) => {
                        match s.cb_sink().send(Box::new(|s| {
                            let d = Dialog::around(TextView::new("The instance will reboot."))
                                .title("Reboot instance")
                                .button("Cancel", |s| {
                                    s.pop_layer();
                                });

                            let dl = event_view(d);

                            s.add_layer(dl);
                        })) {
                            Ok(_) => (),
                            Err(err) => {
                                s.pop_layer();
                                error_dialog(s, "Could not reboot instance.", &format!("{}", err));
                            }
                        };
                    }
                    Err(err) => {
                        s.pop_layer();
                        error_dialog(s, "Could not reboot instance.", &format!("{}", err));
                    }
                };
            }
        }
    }
    select.set_on_submit(ok);

    let table = &s
        .find_name::<InstancesView<Instance, BasicColumn>>("instances")
        .unwrap();

    let instance = table.item();

    if instance.is_none() {
        return;
    }

    let instance = instance.unwrap();

    let select = OnEventView::new(select);
    s.add_layer(event_view(
        Dialog::around(select.scrollable())
            .h_align(HAlign::Center)
            .title(format!(
                "Action ({})",
                &instance.instance.instance_id.clone().unwrap()
            ))
            .button("Cancel", |s| {
                s.pop_layer();
            }),
    ));
}

fn change_region(s: &mut Cursive) {
    let regions = vec![
        ApEast1,
        AfSouth1,
        ApNortheast1,
        ApNortheast2,
        ApSouth1,
        ApSoutheast1,
        ApSoutheast2,
        CaCentral1,
        EuCentral1,
        EuWest1,
        EuWest2,
        EuWest3,
        EuNorth1,
        EuSouth1,
        SaEast1,
        UsEast1,
        UsEast2,
        UsWest1,
        UsWest2,
        UsGovEast1,
        UsGovWest1,
        CnNorth1,
        CnNorthwest1,
    ];

    let mut select = SelectView::<String>::new()
        // Center the text horizontally
        .h_align(HAlign::Center)
        // Use keyboard to jump to the pressed letters
        .autojump();

    select.add_all_str(regions.iter().map(|r| r.name()));

    let ud = s.user_data::<ReturnValues>().unwrap();

    // TODO: change to support multi checkbox, need modified select view
    // currently defaulting to first
    let idx = &select
        .iter()
        .position(|item| item.0 == ud.regions[0].name())
        .unwrap();

    let mut select = select.selected(*idx);

    fn ok(s: &mut Cursive, name: &str) {
        let region = Region::from_str(name).unwrap();

        let ud = s.user_data::<ReturnValues>().unwrap();

        if ud.regions[0].name() == name {
            s.pop_layer();
            return;
        }

        match get_instances_with_region(ud.profiles.clone(), [region.clone()].to_vec()) {
            Ok(instances) => {
                let mut iv = s
                    .find_name::<InstancesView<Instance, BasicColumn>>("instances")
                    .unwrap();
                iv.set_instances(instances.clone());

                s.with_user_data(|v: &mut ReturnValues| {
                    v.regions = [region.clone()].to_vec();
                    v.instances = instances;
                });

                s.pop_layer();

                update_bottom_bar(s);
            }
            Err(err) => {
                let d = Dialog::around(TextView::new(format!(
                    "Could not retrieve instances\n\n{}",
                    err
                )))
                .title("Error")
                .button("Cancel", |s| {
                    s.pop_layer();
                });

                let dl = event_view(d);

                s.add_layer(dl);
            }
        }
    }

    // Sets the callback for when "Enter" is pressed.
    select.set_on_submit(ok);

    // Let's override the `j` and `k` keys for navigation
    let select = OnEventView::new(select)
        .on_pre_event_inner('k', |s, _| {
            s.select_up(1);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner('j', |s, _| {
            s.select_down(1);
            Some(EventResult::Consumed(None))
        });

    s.add_layer(
        OnEventView::new(
            Dialog::around(select.scrollable().fixed_size((20, 10)))
                .title("Switch region")
                .button("Cancel", |s| {
                    s.pop_layer();
                }),
        )
        .on_event(Event::Key(Key::Esc), |s| {
            s.pop_layer();
        }),
    );
}

fn error_dialog(s: &mut Cursive, title: &str, description: &str) {
    let d = Dialog::around(TextView::new(description))
        .title(title)
        .button("Cancel", |s| {
            s.pop_layer();
        });

    let dl = event_view(d);

    s.add_layer(dl);
}

fn change_profile(s: &mut Cursive) {
    fn ok(s: &mut Cursive, name: &str) {
        s.call_on_name("select", |view: &mut SelectView<String>| {
            view.add_item_str(name)
        });
        s.pop_layer();
    }

    s.add_layer(
        Dialog::around(
            EditView::new()
                .on_submit(ok)
                .with_name("name")
                .fixed_width(10),
        )
        .title("Pick a region")
        .button("Ok", |s| {
            let name = s
                .call_on_name("name", |view: &mut EditView| view.get_content())
                .unwrap();
            ok(s, &name);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}

fn ec2_describe_input() -> DescribeInstancesRequest {
    DescribeInstancesRequest {
        ..Default::default()
    }
}
