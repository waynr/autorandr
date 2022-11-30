use xrandr::XHandle;

use crate::{Monitor, Result};

#[cfg_attr(test, faux::create)]
pub(crate) struct XHandleWrapper(XHandle);

impl XHandleWrapper {
    pub(crate) fn open() -> Result<XHandleWrapper> {
        Ok(XHandleWrapper(XHandle::open()?))
    }
}

#[cfg_attr(test, faux::methods)]
impl XHandleWrapper {
    pub fn active_outputs(&mut self) -> Result<Vec<Monitor>> {
        Ok(self
            .0
            .monitors()?
            .iter()
            .flat_map(|xmonitor| xmonitor.outputs.iter().map(|xoutput| xoutput.try_into()))
            .collect::<Result<Vec<Monitor>>>()?)
    }

    pub fn inactive_outputs(&mut self) -> Result<Vec<Monitor>> {
        Ok(self
            .0
            .all_outputs()?
            .iter()
            .map(|xoutput| xoutput.try_into())
            .collect::<Result<Vec<Monitor>>>()?)
    }
}
