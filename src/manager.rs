use std::collections::{HashMap, HashSet};
use std::str;

use subprocess::{Exec, ExitStatus, Redirection};
use xrandr::XHandle;

use crate::{Config, Error, Monitor, Result};

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
                log::info!(" name: {0}", monitor.output_name.as_ref().unwrap());
                log::info!(" edid: {0}", monitor.edid.as_ref().unwrap());
            }
        }

        if self.connected.len() > 0 {
            log::info!("");
            log::info!("connected (inactive):");
            for (_, monitor) in &self.connected {
                log::info!(" name: {}", monitor.output_name.as_ref().unwrap());
                log::info!(" edid: {0}", monitor.edid.as_ref().unwrap());
            }
        }

        if self.disconnected.len() > 0 {
            log::info!("");
            log::info!("disconnected:");
            for monitor in &self.disconnected {
                log::info!(" name: {}", monitor.output_name.as_ref().unwrap());
            }
        }
    }

    pub fn reconcile(&self) -> Result<()> {
        let mut cmd = Exec::cmd("xrandr").stderr(Redirection::Merge);
        for monitor in &self.disconnected {
            match &monitor.output_name {
                Some(name) => {
                    cmd = cmd.arg("--output").arg(&name).arg("--off");
                }
                None => (),
            };
        }

        let mut available: HashSet<String> = HashSet::new();
        for (edid, _) in &self.active {
            available.insert(edid.clone());
        }

        for (edid, _) in &self.connected {
            available.insert(edid.clone());
        }

        for profile in &self.config.profiles {
            log::trace!("meow2");
            if profile.is_available(&available) {
                for (_, profile_monitor) in &profile.monitors {
                    log::trace!("meow");
                    let monitor = match (
                        self.active.get(profile_monitor.edid.as_ref().unwrap()),
                        self.connected.get(profile_monitor.edid.as_ref().unwrap()),
                    ) {
                        (Some(m), None) => m,
                        (None, Some(m)) => m,
                        (Some(_), Some(_)) => {
                            // logically this should be unreachable given how Manager.detect()
                            // populates Manager.active and Manager.connected
                            unreachable!("meow");
                        }
                        (None, None) => {
                            // we shouldn't be able to reach this point since the
                            // profile_monitor.edid is confirmed to be among the `available` edids
                            // in the profile.is_available() method
                            unreachable!("meow");
                        }
                    };
                    cmd = cmd
                        .arg("--output")
                        .arg(monitor.output_name.as_ref().unwrap());
                    log::debug!("{:?}", profile_monitor.get_args());
                    cmd = cmd.args(&profile_monitor.get_args());
                }
                break;
            }
        }

        let cmdline = cmd.to_cmdline_lossy();
        let capture_data = cmd.capture()?;
        match capture_data.exit_status {
            ExitStatus::Exited(0) => {
                log::info!("'{}' succeeded", cmdline);
                Ok(())
            }
            ExitStatus::Exited(s) => {
                log::debug!("{}", str::from_utf8(&capture_data.stderr)?);
                Err(Error::SubprocessFailed(cmdline, s))
            }
            ExitStatus::Signaled(s) => Err(Error::SubprocessKilledBySignal(cmdline, s)),
            _ => Err(Error::SubprocessUnknownFailure(cmdline)),
        }
    }
}
