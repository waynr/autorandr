use xrandr::XHandle;

use crate::{Output, Result};

#[cfg_attr(test, faux::create)]
pub(crate) struct XHandleWrapper(XHandle);

impl XHandleWrapper {
    pub(crate) fn open() -> Result<XHandleWrapper> {
        Ok(XHandleWrapper(XHandle::open()?))
    }
}

#[cfg_attr(test, faux::methods)]
impl XHandleWrapper {
    pub fn active_outputs(&mut self) -> Result<Vec<Output>> {
        Ok(self
            .0
            .monitors()?
            .iter()
            .flat_map(|m| m.outputs.iter().map(|xoutput| xoutput.into()))
            .collect::<Vec<Output>>())
    }

    pub fn inactive_outputs(&mut self) -> Result<Vec<Output>> {
        Ok(self
            .0
            .all_outputs()?
            .iter()
            .map(|o| o.into())
            .collect::<Vec<Output>>())
    }
}
