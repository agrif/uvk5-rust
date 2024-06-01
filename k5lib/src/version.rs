/// Max size of version string, including terminating `NUL`.
pub const VERSION_LEN: usize = 16;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Default)]
pub struct Version([u8; VERSION_LEN]);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum VersionError {
    TooLong,
}

#[cfg(feature = "std")]
impl std::error::Error for VersionError {}

impl core::fmt::Display for VersionError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            VersionError::TooLong => write!(
                f,
                "version must be {} bytes or less, including NUL",
                VERSION_LEN
            ),
        }
    }
}

impl Version {
    pub const fn new_empty() -> Self {
        Self([0; VERSION_LEN])
    }

    pub const fn new(data: [u8; VERSION_LEN]) -> Self {
        Self(data)
    }

    pub const fn new_from_str(name: &str) -> Result<Self, VersionError> {
        Self::new_from_bytes(name.as_bytes())
    }

    pub const fn new_from_c_str(name: &core::ffi::CStr) -> Result<Self, VersionError> {
        Self::new_from_bytes(name.to_bytes())
    }

    pub const fn new_from_bytes(bytes: &[u8]) -> Result<Self, VersionError> {
        // use >= to force space for a terminating 0
        if bytes.len() >= VERSION_LEN {
            return Err(VersionError::TooLong);
        }

        let mut data = [0; VERSION_LEN];
        let mut i = 0;
        while i < bytes.len() && bytes[i] > 0 {
            data[i] = bytes[i];
            i += 1;
        }

        Ok(Self(data))
    }

    pub fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
        // unwrap: always at least one element
        let mut zero = 0;
        while self.0[zero] > 0 && zero < self.0.len() {
            zero += 1;
        }
        let zero_terminated = &self.0[..zero];
        core::str::from_utf8(zero_terminated)
    }

    pub const fn as_c_str(&self) -> Result<&core::ffi::CStr, core::ffi::FromBytesUntilNulError> {
        core::ffi::CStr::from_bytes_until_nul(&self.0)
    }

    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl core::fmt::Debug for Version {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        match self.as_str() {
            Ok(s) => f.debug_tuple("Version").field(&s).finish(),
            Err(_) => f.debug_tuple("Version").field(&self.as_bytes()).finish(),
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Version {
    fn format(&self, f: defmt::Formatter) {
        match self.as_str() {
            Ok(s) => defmt::write!(f, "Version({})", s),
            Err(_) => defmt::write!(f, "Version({})", self.as_bytes()),
        }
    }
}

impl core::ops::Deref for Version {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}
