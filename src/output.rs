use std::collections::BTreeMap;
use std::fmt;

use hex::encode;
use serde::{Deserialize, Serialize};
use xrandr::{Output as XRandrOutput, Value};

/// A display device representation.
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Output {
    pub output_name: Option<String>,
    // TODO: make edid value an enum with variants that allow for multiple possible monitors in
    // profiles (allows a profile with some fixed screens, some dynamic -- eg, the position of one
    // output could be one of multiple outputs)
    pub edid: Option<String>,
    pub xrandr_args: Option<BTreeMap<String, String>>,
}

impl Output {
    pub fn get_args(&self) -> Vec<String> {
        if let Some(args) = &self.xrandr_args {
            args.iter()
                .map(|(k, v)| [k.clone(), v.clone()])
                .flat_map(|s| s)
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl From<&XRandrOutput> for Output {
    fn from(o: &XRandrOutput) -> Output {
        Output {
            output_name: Some(String::from(o.name.clone())),
            edid: match o.properties.get("EDID") {
                Some(p) => match &p.value {
                    Value::Edid(v) => Some(encode(v)),
                    _ => None,
                },
                None => None,
            },
            xrandr_args: Some(BTreeMap::new()),
        }
    }
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(args) = &self.xrandr_args {
            for (arg, value) in args {
                write!(f, "  {0} = {1}\n", arg, value)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use indexmap::IndexMap;
    use xrandr::{Output as XOutput, Property, PropertyValue};

    #[test]
    fn convert_xrandr_output_to_autorandr_output() {
        let edid: Vec<u8> = Vec::from([0]);
        let xo = &XOutput {
            xid: 0,
            name: "MEOW-1".into(),
            properties: IndexMap::from([(
                "EDID".into(),
                Property {
                    name: "EDID".into(),
                    value: PropertyValue::Edid(edid),
                    values: None,
                    is_immutable: true,
                    is_pending: false,
                },
            )]),
        };
        let expected = Output {
            output_name: Some(xo.name.clone()),
            edid: Some("00".into()),
            xrandr_args: BTreeMap::new(),
        };
        let actual: Output = xo.into();
        assert_eq!(expected, actual);
    }
}
