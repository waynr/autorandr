use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

use sha2::{Digest, Sha256};
use xrandr::{Output, PropertyValue};
use serde::{Deserialize, Serialize};

use crate::errors::{Error, Result};

/// A display device representation.
#[derive(Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Monitor {
    pub name: String,
    pub output_name: Option<String>,
    pub edid_digest: Option<Vec<u8>>,
    pub xrandr_fields: BTreeMap<String, String>,
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
            xrandr_fields: BTreeMap::new(),
        }
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

