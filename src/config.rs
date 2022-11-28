use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

use dirs::config_dir;
use serde::{Deserialize, Serialize};

use crate::errors::{Error, Result};
use crate::monitor::Monitor;

/// Representation of a known collection of devices.
#[derive(Deserialize, Serialize, Eq, PartialEq)]
pub struct Profile {
    name: String,
    pub(crate) monitors: BTreeMap<String, Monitor>,

    #[serde(skip)]
    set: HashSet<String>,
}

// public methods
impl Profile {
    pub fn is_available(&self, available_edids: &HashSet<String>) -> bool {
        log::debug!("{:?}", self.set);
        self.set.is_subset(available_edids)
    }
}

// private methods
impl Profile {
    fn init_set(&mut self) {
        for (_, monitor) in &self.monitors {
            if let Some(edid) = &monitor.edid {
                self.set.insert(edid.clone());
            }
        }
    }
}

impl TryFrom<fs::DirEntry> for Profile {
    type Error = Error;

    fn try_from(de: fs::DirEntry) -> Result<Profile> {
        let path: PathBuf = de.path().into();
        match path.extension() {
            Some(ext) if ext == "yaml" || ext == "yml" => {
                let mut file = fs::File::open(path)?;
                let mut contents = String::new();
                let _ = file.read_to_string(&mut contents)?;
                let mut p: Self = serde_yaml::from_str(&contents)?;
                p.init_set();
                Ok(p)
            }
            _ => Err(Error::UnrecognizedProfileConfigFile(path)),
        }
    }
}

impl Ord for Profile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Profile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Config contains profiles (in order of preference) and known monitors.
pub struct Config {
    pub profiles: Vec<Profile>,
    pub monitors: Vec<Monitor>,
}

impl Config {
    pub fn load() -> Result<Config> {
        let mut profiles = fs::read_dir(Config::profiles_dir()?)?
            .filter_map(|entry| {
                log::debug!("{:?}", entry);
                match entry {
                    Ok(e) => match e.try_into() {
                        Ok(e) => Some(e),
                        Err(e) => {
                            log::debug!("{:?}", e);
                            None
                        }
                    },
                    _ => None,
                }
            })
            .collect::<Vec<Profile>>();

        let mut monitors = fs::read_dir(Config::monitors_dir()?)?
            .filter_map(|entry| match entry {
                Ok(e) => match e.try_into() {
                    Ok(e) => Some(e),
                    _ => None,
                },
                _ => None,
            })
            .collect::<Vec<Monitor>>();

        profiles.sort();
        monitors.sort();

        Ok(Config { profiles, monitors })
    }

    fn profiles_dir() -> Result<PathBuf> {
        let dir = config_dir()
            .ok_or(Error::CannotDetermineConfigDir)?
            .join("autorandr")
            .join("profiles");
        fs::create_dir_all(&dir)?;

        Ok(dir)
    }

    fn monitors_dir() -> Result<PathBuf> {
        let dir = config_dir()
            .ok_or(Error::CannotDetermineConfigDir)?
            .join("autorandr")
            .join("monitors");
        fs::create_dir_all(&dir)?;

        Ok(dir)
    }
}
