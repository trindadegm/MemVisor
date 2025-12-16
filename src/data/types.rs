use std::{fmt::Display, str::FromStr};

use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, SerializeDisplay, DeserializeFromStr)]
pub struct DebugPointer(usize);

impl DebugPointer {
    pub const fn new(data: usize) -> Self {
        Self(data)
    }

    pub const fn null() -> Self {
        Self(0)
    }

    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    pub const fn as_usize(self) -> usize {
        self.0
    }
}
impl Display for DebugPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}
impl FromStr for DebugPointer {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let number = usize::from_str_radix(s.trim_start_matches("0x"), 16)?;

        Ok(Self(number))
    }
}
