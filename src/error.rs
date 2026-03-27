use crate::DshCliResult;
use dsh_api::error::DshApiError;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone)]
pub(crate) enum DshCliError {
  AesGcm(String),
  Canceled,
  Configuration(String),
  Conversion(String),
  Decode(String),
  Discovery(String),
  DshApi(String),
  Home(String),
  Io(String),
  Keyring(String),
  Rcgen(String),
  Reqwest(String),
  SerdeJson(String),
  String(String),
  Time(String),
  TokioJoin(String),
  UrlParse(String),
  Utf8(String),
  Whoami(String),
  _X509(String),
}

impl DshCliError {
  pub(crate) fn accept_not_found(error: DshApiError, print: impl Fn()) -> DshCliResult<()> {
    match error {
      DshApiError::NotFound { message: None } => {
        print();
        Ok(())
      }
      error => Err(DshCliError::from(error)),
    }
  }
}

impl Error for DshCliError {}

/// Creates an `Err(DshCliError::String)` with a formatted string.
///
/// The arguments for the `err!` macro are the same as the arguments for the [`format!`] macro.
/// The values must all implement `Display`. The macro will create an
/// [`Err<DshCliError::String>`], where the message will be generated from the format string.
///
/// ## Examples
/// ```
/// fn divide(numerator: i64, denominator: i64) -> Result<i64, DshCliError> {
///   if denominator == 0 {
///     err!("cannot divide {} by zero", numerator)
///   } else {
///     Ok(numerator / denominator)
///   }
/// }
/// assert_eq!(divide(42, 0), DshCliError::String("cannot divide 42 by zero".to_string()));
/// ```
#[macro_export]
macro_rules! err {
  ($($t:tt)*) => {{
    Err($crate::error::DshCliError::String(format!($($t)*)))
  }};
}

/// Creates a `DshCliError::String` with a formatted string.
///
/// The arguments for the `err!` macro are the same as the arguments for the [`format!`] macro.
/// The values must all implement `Display`. The macro will create an
/// [`DshCliError::String`], where the message will be generated from the format string.
///
/// ## Examples
/// ```
/// fn divide(numerator: i64, denominator: i64) -> Result<i64, DshCliError> {
///   if denominator == 0 {
///     Err(error!("cannot divide {} by zero", numerator))
///   } else {
///     Ok(numerator / denominator)
///   }
/// }
/// assert_eq!(divide(42, 0), DshCliError::String("cannot divide 42 by zero".to_string()));
/// ```
#[macro_export]
macro_rules! cli_error {
  ($($t:tt)*) => {{
    $crate::error::DshCliError::String(format!($($t)*))
  }};
}

/// Creates a closure that will map a single argument into an `DshCliError::String`.
///
/// The argument for the `error_map!` macro must be a literal format string, which must contain
/// exactly one `{}` placeholder. The macro will then create a closure that maps a value (which
/// must implement `Display`) to an instance of a [`DshCliError::String`], where the message will
/// be generated from the format string and the value using the `format!` macro.
///
/// The intended use for `error_map!` is as a closure for the [`Result::map_err`] method, mapping
/// any error value (as long as its type implements `Display`) into a `DshCliError::String`.
///
/// ## Examples
/// ```
/// fn save(path: PathBuf, data: &[u8]) -> Result<(), DshCliError> {
///   fs::write(path, data).map_err(error_map!("write failed with error: {}"))
/// }
/// ```
#[macro_export]
macro_rules! error_map {
  ($fmt:literal) => {{
    |error| $crate::error::DshCliError::String(format!($fmt, error))
  }};
}

/// Creates a closure that will map a single argument into an `DshCliError::String`.
///
/// The arguments for the `error_append!` macro are the same as the arguments for the [`format!`]
/// macro. The values must all implement `Display`. The macro will create a closure that maps a
/// value (which must implement `Display`) to an instance of a [`DshCliError::String`], where
/// the message will be generated from the format string and the parameters using the `format!`
/// macro, post-fixed by the closure's argument converted to a
/// `String`.
///
/// The intended use for `error_append!` is as a closure for the [`Result::map_err`] method,
/// mapping any error value (as long as its type implements `Display`) into a `DshCliError::String`.
///
/// ## Examples
/// ```
/// fn save(path: PathBuf, data: &[u8]) -> Result<(), DshCliError> {
///   fs::write(path, data).map_err(error_append!("writing {} failed with error: ", path))
/// }
/// ```
#[macro_export]
macro_rules! error_append {
  ($($t:tt)*) => {{
    |error| $crate::error::DshCliError::String(format!("{}{}", format!($($t)*), error))
  }};
}

impl Debug for DshCliError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::AesGcm(message) => write!(f, "DshCliError(aes gcm, {})", message),
      Self::Canceled => write!(f, "DshCliError(canceled)"),
      Self::Configuration(message) => write!(f, "DshCliError(configuration, {})", message),
      Self::Conversion(message) => write!(f, "DshCliError(conversion, {})", message),
      Self::Decode(message) => write!(f, "DshCliError(decode, {})", message),
      Self::Discovery(message) => write!(f, "DshCliError(discovery, {})", message),
      Self::DshApi(message) => write!(f, "DshCliError(dsh api, {})", message),
      Self::Home(message) => write!(f, "DshCliError(home, {})", message),
      Self::Io(message) => write!(f, "DshCliError(io, {})", message),
      Self::Keyring(message) => write!(f, "DshCliError(ikeyring, {})", message),
      Self::Rcgen(message) => write!(f, "DshCliError(rcgen, {})", message),
      Self::Reqwest(message) => write!(f, "DshCliError(reqwest, {})", message),
      Self::SerdeJson(message) => write!(f, "DshCliError(json, {})", message),
      Self::String(message) => write!(f, "DshCliError({})", message),
      Self::Time(message) => write!(f, "DshCliError(time, {})", message),
      Self::TokioJoin(message) => write!(f, "DshCliError(tokio join, {})", message),
      Self::UrlParse(message) => write!(f, "DshCliError(url parse, {})", message),
      Self::Utf8(message) => write!(f, "DshCliError(utf8, {})", message),
      Self::Whoami(message) => write!(f, "DshCliError(whoami, {})", message),
      Self::_X509(message) => write!(f, "DshCliError(x509, {})", message),
    }
  }
}

impl Display for DshCliError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::AesGcm(message) => write!(f, "{}", message),
      Self::Canceled => f.write_str("canceled"),
      Self::Configuration(message) => write!(f, "{}", message),
      Self::Conversion(message) => write!(f, "{}", message),
      Self::Decode(message) => write!(f, "{}", message),
      Self::Discovery(message) => write!(f, "{}", message),
      Self::DshApi(message) => write!(f, "{}", message),
      Self::Home(message) => write!(f, "{}", message),
      Self::Io(message) => write!(f, "{}", message),
      Self::Keyring(message) => write!(f, "{}", message),
      Self::Rcgen(message) => write!(f, "{}", message),
      Self::Reqwest(message) => write!(f, "{}", message),
      Self::SerdeJson(message) => write!(f, "{}", message),
      Self::String(message) => write!(f, "{}", message),
      Self::Time(message) => write!(f, "{}", message),
      Self::TokioJoin(message) => write!(f, "{}", message),
      Self::UrlParse(message) => write!(f, "{}", message),
      Self::Utf8(message) => write!(f, "{}", message),
      Self::Whoami(message) => write!(f, "{}", message),
      Self::_X509(message) => write!(f, "{}", message),
    }
  }
}

impl From<aes_gcm::Error> for DshCliError {
  fn from(aes_gcm_error: aes_gcm::Error) -> Self {
    Self::AesGcm(aes_gcm_error.to_string())
  }
}

impl From<openidconnect::ConfigurationError> for DshCliError {
  fn from(configuration_error: openidconnect::ConfigurationError) -> Self {
    Self::Configuration(configuration_error.to_string())
  }
}

impl From<base64::DecodeError> for DshCliError {
  fn from(decode_error: base64::DecodeError) -> Self {
    Self::Decode(decode_error.to_string())
  }
}

impl From<openidconnect::DiscoveryError<openidconnect::HttpClientError<openidconnect::reqwest::Error>>> for DshCliError {
  fn from(dsh_discovery_error: openidconnect::DiscoveryError<openidconnect::HttpClientError<openidconnect::reqwest::Error>>) -> Self {
    Self::Discovery(dsh_discovery_error.to_string())
  }
}

impl From<dsh_api::types::error::ConversionError> for DshCliError {
  fn from(conversion_error: dsh_api::types::error::ConversionError) -> Self {
    Self::Conversion(conversion_error.to_string())
  }
}

impl From<DshApiError> for DshCliError {
  fn from(dsh_api_error: dsh_api::error::DshApiError) -> Self {
    Self::DshApi(dsh_api_error.to_string())
  }
}

impl From<keyring::Error> for DshCliError {
  fn from(keyring_error: keyring::Error) -> Self {
    Self::Keyring(keyring_error.to_string())
  }
}

impl From<openidconnect::reqwest::Error> for DshCliError {
  fn from(reqwest_error: openidconnect::reqwest::Error) -> Self {
    Self::Reqwest(reqwest_error.to_string())
  }
}

impl From<serde_json::Error> for DshCliError {
  fn from(serde_json_error: serde_json::Error) -> Self {
    Self::SerdeJson(serde_json_error.to_string())
  }
}

impl From<homedir::GetHomeError> for DshCliError {
  fn from(get_home_error: homedir::GetHomeError) -> Self {
    Self::Home(get_home_error.to_string())
  }
}

impl From<std::io::Error> for DshCliError {
  fn from(io_error: std::io::Error) -> Self {
    Self::Io(io_error.to_string())
  }
}

impl From<String> for DshCliError {
  fn from(message: String) -> Self {
    Self::String(message)
  }
}

impl From<&str> for DshCliError {
  fn from(message: &str) -> Self {
    Self::String(message.to_string())
  }
}

impl From<std::time::SystemTimeError> for DshCliError {
  fn from(system_time_error: std::time::SystemTimeError) -> Self {
    Self::Time(system_time_error.to_string())
  }
}

impl From<tokio::task::JoinError> for DshCliError {
  fn from(join_error: tokio::task::JoinError) -> Self {
    Self::TokioJoin(join_error.to_string())
  }
}

impl From<openidconnect::url::ParseError> for DshCliError {
  fn from(parse_error: openidconnect::url::ParseError) -> Self {
    Self::UrlParse(parse_error.to_string())
  }
}

impl From<rcgen::Error> for DshCliError {
  fn from(rcgen_error: rcgen::Error) -> Self {
    Self::Rcgen(rcgen_error.to_string())
  }
}

impl From<std::string::FromUtf8Error> for DshCliError {
  fn from(utf8_error: std::string::FromUtf8Error) -> Self {
    Self::Utf8(utf8_error.to_string())
  }
}

impl From<whoami::Error> for DshCliError {
  fn from(whoami_error: whoami::Error) -> Self {
    Self::Whoami(whoami_error.to_string())
  }
}

impl From<DshCliError> for String {
  fn from(dsh_cli_error: DshCliError) -> Self {
    dsh_cli_error.to_string()
  }
}
