use std::collections::{HashMap, HashSet};
use std::str;

use subprocess::{Exec, ExitStatus, Redirection};

use crate::xhandle::XHandleWrapper;
use crate::{Config, Error, Output, Result};

pub struct Manager {
    config: Config,
    xhandle: XHandleWrapper,

    active: HashMap<String, Output>,
    connected: HashMap<String, Output>,
    disconnected: Vec<Output>,
}

impl Manager {
    pub fn from(config: Config) -> Result<Self> {
        Ok(Manager {
            config,
            xhandle: XHandleWrapper::open()?,
            active: HashMap::new(),
            connected: HashMap::new(),
            disconnected: Vec::new(),
        })
    }

    pub fn detect(mut self) -> Result<Self> {
        self.active = HashMap::new();
        self.connected = HashMap::new();
        self.disconnected = Vec::new();

        for o in self.xhandle.active_outputs()? {
            if let Some(edid) = &o.edid {
                self.active.insert(edid.to_string(), o);
            }
        }

        for o in self.xhandle.inactive_outputs()? {
            match (&o.output_name, &o.edid) {
                (Some(_), Some(edid)) => {
                    if self.active.contains_key(edid) {
                        continue;
                    }
                    self.connected.insert(edid.to_string(), o);
                }
                (Some(_), None) => {
                    self.disconnected.push(o);
                }
                (_, _) => continue,
            }
        }

        Ok(self)
    }

    pub fn list(&self) {
        if self.active.len() > 0 {
            log::info!("connected (active):");
            for (_, output) in &self.active {
                log::info!(" name: {0}", output.output_name.as_ref().unwrap());
                log::info!(" edid: {0}", output.edid.as_ref().unwrap());
            }
        }

        if self.connected.len() > 0 {
            log::info!("");
            log::info!("connected (inactive):");
            for (_, output) in &self.connected {
                log::info!(" name: {}", output.output_name.as_ref().unwrap());
                log::info!(" edid: {0}", output.edid.as_ref().unwrap());
            }
        }

        if self.disconnected.len() > 0 {
            log::info!("");
            log::info!("disconnected:");
            for output in &self.disconnected {
                log::info!(" name: {}", output.output_name.as_ref().unwrap());
            }
        }
    }

    pub fn profiles(&self) {
        log::info!("available profiles:");
        for profile in &self.config.profiles {
            log::info!("{0}", profile);
        }
    }

    pub fn reconcile(&self) -> Result<()> {
        let mut cmd = Exec::cmd("xrandr").stderr(Redirection::Merge);
        for output in &self.disconnected {
            match &output.output_name {
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
            if profile.is_available(&available) {
                for (_, profile_output) in &profile.outputs {
                    let output = match (
                        self.active.get(profile_output.edid.as_ref().unwrap()),
                        self.connected.get(profile_output.edid.as_ref().unwrap()),
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
                            // profile_output.edid is confirmed to be among the `available` edids
                            // in the profile.is_available() method
                            unreachable!("meow");
                        }
                    };
                    cmd = cmd
                        .arg("--output")
                        .arg(output.output_name.as_ref().unwrap());
                    log::debug!("{:?}", profile_output.get_args());
                    cmd = cmd.args(&profile_output.get_args());
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

    pub fn mirror(&self) -> Result<()> {
        let mut cmd = Exec::cmd("xrandr").stderr(Redirection::Merge);
        for output in &self.disconnected {
            match &output.output_name {
                Some(name) => {
                    cmd = cmd.arg("--output").arg(&name).arg("--off");
                }
                None => (),
            };
        }

        let mirror_profile_output = self
            .config
            .profiles
            .iter()
            .find_map(|p| {
                if p.name() == "mirror" {
                    Some(p.outputs.get("all-monitors").unwrap())
                } else {
                    None
                }
            })
            .ok_or(Error::MirrorModeMissingProfile)?;

        let mut actives = self.active.iter();
        let active = if let Some((_edid, output)) = actives.next() {
            output
        } else {
            return Err(Error::NoActiveMonitors);
        };

        cmd = cmd
            .arg("--output")
            .arg(active.output_name.as_ref().unwrap());
        log::debug!("{:?}", mirror_profile_output.get_args());
        cmd = cmd.args(&mirror_profile_output.get_args());

        if actives.next().is_some() {
            return Err(Error::MirrorModeTooManyActiveMonitors);
        }

        for (_edid, output) in &self.connected {
            // TODO: fail if all available monitors do not have the same resolution as the current active
            // monitor
            cmd = cmd
                .arg("--output")
                .arg(output.output_name.as_ref().unwrap());
            log::debug!("{:?}", mirror_profile_output.get_args());
            cmd = cmd.args(&mirror_profile_output.get_args());
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
