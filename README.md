# Cloudman
----------
Cloudman is a textual user interface (heavily inspired by htop) to manage your Amazon EC2 fleet instantly. By using Cloudman you'll find an overview of your instances, navigate through regions, retrieve instance details, show console outputs and connect to instance terminal using SSM.

The profiles and defaults as configured in ~/.aws/credentials will be used. 

[![Build status](https://api.travis-ci.org/dutchcoders/cloudman.svg?branch=master&status=passed)](https://travis-ci.org/github/dutchcoders/cloudman)
[![Crates.io](https://img.shields.io/crates/v/cloudman.svg)](https://crates.io/crates/cloudman)
[![Packaging status](https://repology.org/badge/tiny-repos/cloudman.svg)](https://repology.org/project/cloudman/badges)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

### Screenshots
<div>
<img src="screenshots/instances.png" width="48%" title="instances overview" />
<img src="screenshots/change_region.png" width="48%" title="change aws region" />
<img src="screenshots/details.png" width="48%" title="detail information for instance" />
<img src="screenshots/console.png" width="48%" title="console output for instance" />
</div>

## Usage
Cloudman can be started optionally with a region and profile to use. 

```
cloudman-rs 0.1.7
Remco Verhoef <remco@dutchcoders.io>

USAGE:
    cloudman [FLAGS] [OPTIONS]

FLAGS:
        --disable-dry-run    Disable dry run
    -h, --help               Prints help information
        --use-env            Usen environment credentials
    -V, --version            Prints version information

OPTIONS:
    -p, --profile <profile>...    One or more profiles to use
    -r, --region <region>...      One or more regions to use
```

## Shortcuts

| Shortcut  | Description |
| ------------- | ------------- |
| F1 | display help |
| F2 | connect using [ssm](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/session-manager.html) to instance |
| F3 | search through displayed instances |
| F4 | filter displayed instances |
| F5 | refresh displayed instances |
| F6 | show actions for instances |
| F7 | switch region |
| L  | display console output for instance |
| ESC  | close window |
| Q  | quit |

## Installation

### Homebrew
If you're a **macOS Homebrew** or a **Linuxbrew** user, then you can install
cloudman from homebrew tap:

```bash
brew install dutchcoders/cloudman/cloudman
```

### MacPorts
If you're a **MacPorts** user, you can install Cloudman using:

```bash
sudo port selfupdate
sudo port install cloudman
```

## Building
cloudman is written in Rust, so you'll need to grab a
[Rust installation](https://www.rust-lang.org/) in order to compile it.
cloudman compiles with Rust 1.44.0 (stable) or newer. In general, cloudman tracks
the latest stable release of the Rust compiler.

To build cloudman:

```
$ git clone https://github.com/dutchcoders/cloudman
$ cd cloudman
$ cargo build --release
$ ./target/release/cloudman --version
0.1.0
```

## Current features
* overview of all instances
* support different profiles
* switch easily between aws regions
* connect using SSM to instance (using tmux)
* search through instances
* filter instances
* show detailed information for instances
* show console output if supported

# Roadmap
* start and stop instances (actions)
* request spot instances
* modifyable columns
* sorting
* show filter active
* show indicator of loading
* search through console output
* much more

## Contribute

+ I :heart: pull requests and bug reports
+ Don't hesitate to [tell me my rust skills suck](https://github.com/dutchcoders/cloudman/issues/new), but please tell me why.

## Thanks

Special thanks to:

* [Doom Emacs](https://github.com/hlissner/doom-emacs) for making the Emacs framework I love.
* [Cursive](https://github.com/gyscos/cursive/) for making the textual user interface Cloudman is built upon.
* [htop](https://github.com/hishamhm/htop) for the inspirational interface.

Everyone else that inspired me.

## Creator

**Remco Verhoef**

- <https://twitter.com/remco_verhoef>
- <https://twitter.com/dutchcoders>


## Copyright and license

Code and documentation copyright 2011-2020 Remco Verhoef.

Code released under [the MIT license](LICENSE).
