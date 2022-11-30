use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

use hex::encode;
use serde::{Deserialize, Serialize};
use xrandr::{Output as XRandrOutput, PropertyValue};

use crate::errors::{Error, Result};

/// A display device representation.
#[derive(Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Output {
    pub output_name: Option<String>,
    // TODO: make edid value an enum with variants that allow for multiple possible monitors in
    // profiles (allows a profile with some fixed screens, some dynamic -- eg, the position of one
    // output could be one of multiple outputs)
    pub edid: Option<String>,
    pub xrandr_args: BTreeMap<String, String>,
}

impl Output {
    pub fn get_args(&self) -> Vec<String> {
        self
            .xrandr_args
            .iter()
            .map(|(k, v)| [k.clone(), v.clone()])
            .flat_map(|s| s)
            .collect()
    }
}

impl TryFrom<&XRandrOutput> for Output {
    type Error = Error;

    fn try_from(o: &XRandrOutput) -> Result<Output> {
        Ok(Output {
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

impl TryFrom<fs::DirEntry> for Output {
    type Error = Error;

    fn try_from(de: fs::DirEntry) -> Result<Output> {
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
