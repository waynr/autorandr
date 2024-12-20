use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

use dirs::config_dir;
use serde::{Deserialize, Serialize};

use crate::errors::{Error, Result};
use crate::output::Output;

/// Representation of a known collection of devices.
#[derive(Deserialize, Serialize, Eq, PartialEq)]
pub struct Profile {
    pub(crate) outputs: BTreeMap<String, Output>,
    pub(crate) profile_name: Option<String>,

    #[serde(skip)]
    name: String,
    #[serde(skip)]
    set: HashSet<String>,
}

// public methods
impl Profile {
    pub fn is_available(&self, available_edids: &HashSet<String>) -> bool {
        log::debug!("{:?}", self.set);
        self.set.is_subset(available_edids)
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

// private methods
impl Profile {
    fn init_set(&mut self) {
        for (_, output) in &self.outputs {
            if let Some(edid) = &output.edid {
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
                let mut file = fs::File::open(&path)?;
                let mut contents = String::new();
                let _ = file.read_to_string(&mut contents)?;
                let mut p: Self = serde_yaml::from_str(&contents)?;
                p.init_set();
                if let Some(ref s) = p.profile_name {
                    p.name = s.clone();
                } else if let Some(s) = path.file_stem() {
                    p.name = String::from(s.to_str().unwrap());
                }
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

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n", self.name)?;
        for (name, output) in &self.outputs {
            write!(f, "{}\n", name)?;
            write!(f, "{}", output)?;
        }
        Ok(())
    }
}

/// Config contains profiles (in order of preference) and known outputs.
pub struct Config {
    pub profiles: Vec<Profile>,
}

impl Config {
    pub fn load() -> Result<Config> {
        let mut profiles = fs::read_dir(Config::profiles_dir()?)?
            .filter_map(|entry| match entry {
                Ok(entry) => {
                    let path = entry.path();
                    let path_str = path.to_string_lossy();
                    log::debug!("loading {}", &path_str);
                    match entry.try_into() {
                        Ok(e) => Some(e),
                        Err(e) => {
                            log::warn!("failed to load {}:\n{:?}", &path_str, e);
                            None
                        }
                    }
                }
                _ => None,
            })
            .collect::<Vec<Profile>>();

        profiles.sort();
        log::debug!("profiles loaded:");
        for profile in &profiles {
            log::debug!("  {0}", profile.name);
        }

        Ok(Config { profiles })
    }

    fn profiles_dir() -> Result<PathBuf> {
        let dir = config_dir()
            .ok_or(Error::CannotDetermineConfigDir)?
            .join("autorandr")
            .join("profiles");
        fs::create_dir_all(&dir)?;

        Ok(dir)
    }
}
