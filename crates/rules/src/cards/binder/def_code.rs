use std::fmt;

/// Names a card definition across binders: its set's prefix, its name and its version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DefCode {
    pub set: &'static str,
    pub name: &'static str,
    pub version: u16,
}

impl fmt::Display for DefCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.v{}", self.set, self.name, self.version)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn a_code_displays_as_its_set_name_and_version_joined_by_dots() {
        let code = DefCode {
            set: "test",
            name: "bolt",
            version: 0,
        };

        assert_eq!(code.to_string(), "test.bolt.v0");
    }
}
