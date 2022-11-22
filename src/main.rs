use std::collections::{BTreeMap, HashMap};
use std::process::Command;

use clap;
use sha2::{Digest, Sha256};
use xrandr::{Output, PropertyValue, XHandle};

use autorandr::Result;

enum DeviceKind {
    Laptop,
    Monitor,
    Unknown,
}

// A device representation.
struct Monitor {
    name: String,
    output_name: Option<String>,
    edid_digest: Option<Vec<u8>>,
    kind: DeviceKind,
    xrandr_fields: BTreeMap<String, String>,
}

impl From<&Output> for Monitor {
    fn from(o: &Output) -> Monitor {
        Monitor {
            name: "".into(),
            output_name: Some(String::from(o.name.clone())),
            edid_digest: match o.properties.get("EDID") {
                Some(p) => match &p.value {
                    PropertyValue::Edid(v) => {
                        let mut hasher = Sha256::new();
                        hasher.update(v);
                        let digest = hasher.finalize().to_vec();
                        log::info!("{:?}", digest);
                        Some(digest)
                    }
                    _ => None,
                },
                None => None,
            },
            kind: DeviceKind::Unknown,
            xrandr_fields: BTreeMap::new(),
        }
    }
}

// Representation of a knwown collection of devices.
struct Profile<'a> {
    name: String,
    device: BTreeMap<String, &'a Monitor>,
}

fn known_profiles<'a>() -> Result<BTreeMap<String, Profile<'a>>> {
    return Ok(BTreeMap::new());
}

fn known_monitors() -> Result<HashMap<String, Monitor>> {
    Ok(HashMap::from([
        (
            "left-monitor".into(),
            Monitor {
                name: "left-monitor".into(),
                edid_digest: Some(hex::decode(
                    "020d8c7e6dd847d2296b1706d96a75bd939a2b8db86687916fa4d5063da096fe",
                )?),
                output_name: None,
                kind: DeviceKind::Monitor,
                xrandr_fields: BTreeMap::from([("--rotate", "normal"), ("--dpi", "192")].map(|(k, v)| (String::from(k), String::from(v)))),
            },
        ),
        (
            "right-monitor".into(),
            Monitor {
                name: "right-monitor".into(),
                edid_digest: Some(hex::decode(
                    "fc4693d8e06355a504397b5ef480fc29930192606637e3b1ae2c5226b3f0befd",
                )?),
                output_name: None,
                kind: DeviceKind::Monitor,
                xrandr_fields: BTreeMap::from([("--rotate", "normal"), ("--dpi", "192")].map(|(k, v)| (String::from(k), String::from(v)))),
            },
        ),
        // TODO:
    ]))
}

fn list() -> Result<clap::Command> {
    Ok(clap::Command::new("list").about("list profiles and displays"))
}

fn cli() -> Result<clap::ArgMatches> {
    let list = list()?;
    Ok(clap::Command::new("autorandr")
        .author("wayne warren")
        .version("0.0.1")
        .about("xrandr, automatically")
        .args([clap::Arg::new("verbose")
            .short('v')
            .help("verbosity")
            .action(clap::ArgAction::Count)])
        .subcommands(vec![list])
        .get_matches())
}

fn main() -> Result<()> {
    let matches = cli()?;
    let mut logger_builder = &mut pretty_env_logger::formatted_builder();

    let verbosity = matches.get_one::<u8>("verbose").copied();
    let level = match verbosity {
        Some(0) => log::LevelFilter::Info,
        Some(1) => log::LevelFilter::Debug,
        Some(_) => log::LevelFilter::Trace,
        None => log::LevelFilter::Info,
    };

    logger_builder = logger_builder.filter_level(level);
    if level == log::LevelFilter::Info {
        logger_builder = logger_builder.default_format();
        logger_builder = logger_builder.format_module_path(false);
        logger_builder = logger_builder.format_level(false);
        logger_builder = logger_builder.format_timestamp(None);
    }

    logger_builder.try_init()?;
    //log::debug!("verbosity set to {0}", level);
    let mut active: Vec<Monitor> = Vec::new();
    let mut connected: Vec<Monitor> = Vec::new();
    let mut disconnected: Vec<Monitor> = Vec::new();

    log::info!("outputs");
    let monitors = XHandle::open()?.monitors()?;

    for monitor in &monitors {
        for output in &monitor.outputs {
            log::info!("active: {0}", output.name);
            active.push(output.into());
        }
    }

    let outputs = XHandle::open()?.all_outputs()?;

    let known_monitors = known_monitors()?;
    for output in outputs {
        match output.properties.get("EDID") {
            Some(_) => {
                let monitor: Monitor = (&output).into();
                connected.push(monitor);
            }
            None => {
                disconnected.push((&output).into());
            }
        }
    }
    log::info!("connected:");
    for monitor in connected {
        log::info!(" {:?}", monitor.output_name);
    }
    log::debug!("disconnected:");
    for monitor in &disconnected {
        log::debug!(" {:?}", monitor.output_name);
    }

    let mut cmd = Command::new("xrandr");
    for monitor in &disconnected {
        match &monitor.output_name {
            Some(name) => {
                let _ = cmd.arg("--output").arg(&name).arg("--off");
            }
            None => (),
        };
    }

    Ok(())
}
