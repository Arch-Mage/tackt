/// Error returned when a route does not match.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    /// Route path does not match.
    Path,
    /// Route method does not match.
    Method,
    /// Route prefix does not match.
    Prefix,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Path => f.write_str("route path does not match"),
            Error::Method => f.write_str("route method does not match"),
            Error::Prefix => f.write_str("route prefix does not match"),
        }
    }
}
