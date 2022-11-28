use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

use hex::encode;
use serde::{Deserialize, Serialize};
use xrandr::{Output, PropertyValue};

use crate::errors::{Error, Result};

#[derive(Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd)]
pub enum MonitorKind {
    Laptop,
    External,
    Unknown,
}

/// A display device representation.
#[derive(Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Monitor {
    pub kind: MonitorKind,
    pub output_name: Option<String>,
    pub edid: Option<String>,
    pub xrandr_args: BTreeMap<String, String>,
}

impl Monitor {
    pub fn get_args(&self) -> Vec<String> {
        self
            .xrandr_args
            .iter()
            .map(|(k, v)| [k.clone(), v.clone()])
            .flat_map(|s| s)
            .collect()
    }
}

impl TryFrom<&Output> for Monitor {
    type Error = Error;

    fn try_from(o: &Output) -> Result<Monitor> {
        Ok(Monitor {
            kind: MonitorKind::Unknown,
            output_name: Some(String::from(o.name.clone())),
            edid: match o.properties.get("EDID") {
                Some(p) => match &p.value {
                    PropertyValue::Edid(v) => Some(encode(v)),
                    _ => None,
                },
                None => None,
            },
            xrandr_args: BTreeMap::new(),
        })
    }
}

impl TryFrom<fs::DirEntry> for Monitor {
    type Error = Error;

    fn try_from(de: fs::DirEntry) -> Result<Monitor> {
        let path: PathBuf = de.path().into();
        match path.extension() {
            Some(ext) if ext == "yaml" || ext == "yml" => {
                let mut file = fs::File::open(path)?;
                let mut contents = String::new();
                let _ = file.read_to_string(&mut contents)?;
                let c: Self = serde_yaml::from_str(&contents)?;
                Ok(c)
            }
            _ => Err(Error::UnrecognizedProfileConfigFile(path)),
        }
    }
}
