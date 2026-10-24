use std::fmt;

macro_rules! define_scalar_u32 {
    (
        $(#[$meta:meta])*
        $name:ident
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub u32);

        impl $name {
            pub const ZERO: Self = Self(0);

            pub const fn new(raw: u32) -> Self {
                Self(raw)
            }

            pub const fn raw(self) -> u32 {
                self.0
            }
        }

        impl From<u32> for $name {
            fn from(value: u32) -> Self {
                Self::new(value)
            }
        }

        impl From<$name> for u32 {
            fn from(value: $name) -> Self {
                value.raw()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

define_scalar_u32!(
    /// Shared 32-bit version token wrapper.
    VersionToken
);

define_scalar_u32!(
    /// Shared 32-bit timestamp wrapper when the owning subsystem defines the epoch/units.
    Timestamp32
);

impl Timestamp32 {
    // Replaces CTimestamp<unsigned long,0> wraparound ordering.
    pub const fn retail_lt(self, rhs: Self) -> bool {
        let lhs = self.raw();
        let rhs = rhs.raw();
        if lhs == rhs {
            return false;
        }

        let (delta, mut sign) = if lhs < rhs {
            (rhs - lhs, -1)
        } else {
            (lhs - rhs, 1)
        };

        if delta > 0x7FFF_FFFF {
            sign = -sign;
        }

        sign < 0
    }

    pub const fn retail_gt(self, rhs: Self) -> bool {
        rhs.retail_lt(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{Timestamp32, VersionToken};

    #[test]
    fn scalar_wrappers_round_trip() {
        let version = VersionToken::new(7);
        let timestamp = Timestamp32::new(123456789);

        assert_eq!(u32::from(version), 7);
        assert_eq!(timestamp.raw(), 123456789);
        assert_eq!(version.to_string(), "7");
        assert_eq!(format!("{timestamp:?}"), "Timestamp32(123456789)");
    }

    #[test]
    fn timestamp32_matches_retail_wraparound_ordering() {
        assert!(!Timestamp32::new(7).retail_lt(Timestamp32::new(7)));
        assert!(Timestamp32::new(6).retail_lt(Timestamp32::new(7)));
        assert!(!Timestamp32::new(7).retail_lt(Timestamp32::new(6)));
        assert!(Timestamp32::new(0xFFFF_FFFE).retail_lt(Timestamp32::new(1)));
        assert!(!Timestamp32::new(0).retail_lt(Timestamp32::new(0x8000_0000)));
        assert!(Timestamp32::new(0x8000_0000).retail_lt(Timestamp32::new(0)));
    }
}
