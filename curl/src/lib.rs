pub use regex;

pub use curl_derive::FromRegex;

#[derive(Debug)]
pub enum Error {
    NoMatch,
    Missing { field: &'static str },
    Parse { field: &'static str, source: String },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoMatch => f.write_str("input did not match the regex"),
            Error::Missing { field } => {
                write!(f, "capture group `{field}` did not participate in the match")
            }
            Error::Parse { field, source } => {
                write!(f, "failed to parse capture group `{field}`: {source}")
            }
        }
    }
}

impl std::error::Error for Error {}