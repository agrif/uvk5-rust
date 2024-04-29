/// Max size of version string, including terminating NUL.
pub const VERSION_LEN: usize = 16;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Version([u8; VERSION_LEN]);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VersionError {
    TooLong,
}

impl std::error::Error for VersionError {}

impl std::fmt::Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
    pub fn new_empty() -> Self {
        Self([0; VERSION_LEN])
    }

    pub fn new(data: [u8; VERSION_LEN]) -> Self {
        Self(data)
    }

    pub fn from_str(name: &str) -> Result<Self, VersionError> {
        Self::from_bytes(name.as_bytes())
    }

    pub fn from_c_str(name: &std::ffi::CStr) -> Result<Self, VersionError> {
        Self::from_bytes(name.to_bytes())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, VersionError> {
        if bytes.len() > VERSION_LEN {
            return Err(VersionError::TooLong);
        }

        let mut data = [0; VERSION_LEN];
        for (i, b) in bytes.iter().enumerate() {
            data[i] = *b;
        }

        Ok(Self(data))
    }

    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        // unwrap: always at least one element
        let zero_terminated = self.0.split(|b| *b == 0).next().unwrap();
        std::str::from_utf8(zero_terminated)
    }

    pub fn as_c_str(&self) -> Result<&std::ffi::CStr, std::ffi::FromBytesUntilNulError> {
        std::ffi::CStr::from_bytes_until_nul(&self.0)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Debug for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self.as_str() {
            Ok(s) => f.debug_tuple("Version").field(&s).finish(),
            Err(_) => f.debug_tuple("Version").field(&self.as_bytes()).finish(),
        }
    }
}

impl std::ops::Deref for Version {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}
