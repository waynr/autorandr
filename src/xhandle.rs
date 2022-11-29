use xrandr::{Monitor, Output, XHandle};

use crate::Result;

#[cfg_attr(test, faux::create)]
pub(crate) struct XHandleWrapper {
    inner: XHandle,
}

impl XHandleWrapper {
    pub(crate) fn open() -> Result<XHandleWrapper> {
        Ok(XHandleWrapper {
            inner: XHandle::open()?,
        })
    }
}

#[cfg_attr(test, faux::methods)]
impl XHandleWrapper {
    pub fn monitors(&mut self) -> Result<Vec<Monitor>> {
        Ok(self.inner.monitors()?)
    }

    pub fn all_outputs(&mut self) -> Result<Vec<Output>> {
        Ok(self.inner.all_outputs()?)
    }
}
