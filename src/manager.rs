use std::collections::HashMap;

use xrandr::XHandle;

use crate::{Config, Monitor, Result};

pub struct Manager {
    config: Config,

    active: HashMap<String, Monitor>,
    connected: HashMap<String, Monitor>,
    disconnected: Vec<Monitor>,
}

impl Manager {
    pub fn from(config: Config) -> Self {
        Manager {
            config,
            active: HashMap::new(),
            connected: HashMap::new(),
            disconnected: Vec::new(),
        }
    }

    pub fn detect(mut self) -> Result<Self> {
        let mut handle = XHandle::open()?;

        self.active = HashMap::new();
        self.connected = HashMap::new();
        self.disconnected = Vec::new();

        for monitor in handle.monitors()? {
            for output in &monitor.outputs {
                let m: Monitor = output.try_into()?;
                if let Some(edid) = &m.edid {
                    self.active.insert(edid.to_string(), m);
                }
            }
        }

        for output in &handle.all_outputs()? {
            let m: Monitor = output.try_into()?;
            match (&m.output_name, &m.edid) {
                (Some(_), Some(edid)) => {
                    if self.active.contains_key(edid) {
                        continue;
                    }
                    self.connected.insert(edid.to_string(), m);
                }
                (Some(_), None) => {
                    self.disconnected.push(m);
                }
                (_, _) => continue,
            }
        }

        Ok(self)
    }

    pub fn list(&self) {
        if self.active.len() > 0 {
            log::info!("connected (active):");
            for (_, monitor) in &self.active {
                log::info!(" {0}", monitor.output_name.as_ref().unwrap());
                log::info!("  edid: {0}", monitor.edid.as_ref().unwrap());
            }
        }

        if self.connected.len() > 0 {
            log::info!("connected (inactive):");
            for (_, monitor) in &self.connected {
                log::info!(" {:?}", monitor.output_name);
            }
        }

        if self.disconnected.len() > 0 {
            log::info!("disconnected:");
            for monitor in &self.disconnected {
                log::info!(" {:?}", monitor.output_name.as_ref().unwrap());
            }
        }
    }
}
