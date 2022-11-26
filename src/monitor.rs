use std::fs;
use std::io::Read;
use std::path::PathBuf;

use xrandr::{Output, PropertyValue};
use serde::{Deserialize, Serialize};
use hex::encode;

use crate::errors::{Error, Result};

/// A display device representation.
#[derive(Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Monitor {
    pub name: String,
    pub output_name: Option<String>,
    pub edid: Option<String>,
}

impl TryFrom<&Output> for Monitor {
    type Error = Error;

    fn try_from(o: &Output) -> Result<Monitor> {
        Ok(Monitor {
            name: "".into(),
            output_name: Some(String::from(o.name.clone())),
            edid: match o.properties.get("EDID") {
                Some(p) => match &p.value {
                    PropertyValue::Edid(v) => {
                        Some(encode(v))
                    }
                    _ => None,
                },
                None => None,
            },
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
            },
            _ => Err(Error::UnrecognizedProfileConfigFile(path)),
        }
    }
}

