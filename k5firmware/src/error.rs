/// Low-rent anyhow.

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Error(alloc::string::String);

pub type Result<T> = core::result::Result<T, Error>;

impl Error {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<E> From<E> for Error
where
    E: core::fmt::Debug,
{
    fn from(value: E) -> Self {
        Error(alloc::format!("{:?}", value))
    }
}

impl core::ops::Deref for Error {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}
