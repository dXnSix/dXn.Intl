use std::fmt;

/// What went wrong making a formatter or reading a value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IntlError {
    /// The text is not a BCP 47 language tag.
    InvalidLocale(String),
    /// The text or float is not a number a formatter can take (NaN, an infinity, malformed text).
    InvalidNumber(String),
    /// Options that do not go together, such as a minimum above a maximum.
    InvalidOptions(String),
    /// A date or time of day outside the calendar, such as February 30th or 25:00.
    InvalidDateTime(String),
    /// Not a three-letter ISO 4217 currency code.
    InvalidCurrency(String),
    /// The locale data for a formatter could not be loaded.
    Data(String),
}

impl fmt::Display for IntlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLocale(tag) => write!(f, "`{tag}` is not a valid language tag"),
            Self::InvalidNumber(text) => {
                write!(f, "`{text}` is not a number that can be formatted")
            }
            Self::InvalidOptions(reason) => write!(f, "invalid options: {reason}"),
            Self::InvalidDateTime(reason) => write!(f, "invalid date or time: {reason}"),
            Self::InvalidCurrency(code) => write!(f, "`{code}` is not an ISO 4217 currency code"),
            Self::Data(reason) => write!(f, "locale data: {reason}"),
        }
    }
}

impl std::error::Error for IntlError {}

impl From<icu_provider::DataError> for IntlError {
    fn from(error: icu_provider::DataError) -> Self {
        Self::Data(error.to_string())
    }
}

/// The result of this crate's fallible calls.
pub type Result<T, E = IntlError> = std::result::Result<T, E>;
