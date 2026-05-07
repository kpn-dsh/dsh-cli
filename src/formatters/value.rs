use crate::context::Context;
use crate::subjects::certificate::format_distinguished_name;
use chrono::{DateTime, Days, Utc};

#[derive(Clone, Debug)]
pub(crate) enum Value {
  DistinguishedName(String),
  Empty,
  Error(String),
  Hide,
  Ignore(String),
  NotApplicable,
  Plain(String),
  Secret,
  Target(String),
  Todo,
  Unreachable,
  Warn(String),
}

/// Creates a `Value::PLain` with a formatted string.
///
/// The arguments for the `err!` macro are the same as the arguments for the [`format!`] macro.
///
/// # Examples
/// ```
/// let id = "my-id";
/// let plain_value = plain!("id:{}", id);
/// assert!(matches!(plain_value, Value::Plain { .. }));
/// assert_eq!(plain_value.to_undecorated_string, "id:my-id"));
/// ```
#[macro_export]
macro_rules! plain {
  ($($t:tt)*) => {{
    $crate::formatters::value::Value::Plain(format!($($t)*))
  }};
}

/// Creates a `Value::Warn` with a formatted string.
///
/// The arguments for the `err!` macro are the same as the arguments for the [`format!`] macro.
///
/// # Examples
/// ```
/// let id = "my-id";
/// let warn_value = warn!("{} not found", id);
/// assert!(matches!(warn_value, Value::Warn { .. }));
/// assert_eq!(warn_value.to_undecorated_string, "my-id not found"));
/// ```
#[macro_export]
macro_rules! warn {
  ($($t:tt)*) => {{
    $crate::formatters::value::Value::Warn(format!($($t)*))
  }};
}

impl Value {
  /// Create `Value` representing date/time.
  ///
  /// # Parameters
  /// * `datetime` - References a `DateTime<Utc>` struct representing the date/time.
  ///
  /// # Returns
  /// * `Value::Plain`
  #[allow(dead_code)]
  pub(crate) fn datetime(datetime: &DateTime<Utc>) -> Self {
    Self::plain(datetime)
  }

  /// Create `Value` representing date/time with expiration check.
  ///
  /// # Parameters
  /// * `datetime` - References a `DateTime<Utc>` struct representing the date/time.
  /// * `days` - Optional number of days in the future for the warning check.
  ///
  /// # Returns
  /// * `Value::Error` - When `datetime` is in the past.
  /// * `Value::Warn` - When `date` is present and `datetime` will expire within `days` days.
  /// * `Value::Plain` - Otherwise.
  pub(crate) fn datetime_expired(datetime: &DateTime<Utc>, days: Option<u64>) -> Self {
    if datetime < &Utc::now() {
      Self::error(datetime)
    } else {
      match days {
        Some(warning_days) => match Utc::now().checked_add_days(Days::new(warning_days)) {
          Some(expiration_warning_date) => {
            if datetime < &expiration_warning_date {
              Self::warn(datetime)
            } else {
              Self::plain(datetime)
            }
          }
          None => plain!("{} (unchecked)", datetime),
        },
        None => Self::plain(datetime),
      }
    }
  }

  /// Create `Value` representing date/time with not-before check.
  ///
  /// # Parameters
  /// * `datetime` - References a `DateTime<Utc>` struct representing the date/time.
  ///
  /// # Returns
  /// * `Value::Plain` - When `datetime` is in the past.
  /// * `Value::Warn` - When `datetime` is now or in the future.
  pub(crate) fn datetime_not_before(datetime: &DateTime<Utc>) -> Self {
    if datetime > &Utc::now() {
      Self::warn(datetime.to_string())
    } else {
      Self::plain(datetime.to_string())
    }
  }

  /// Create `Value` representing a distinguished name.
  ///
  /// # Parameters
  /// * `distinguished_name` - Can be converted into a `String` representing the distinguished name.
  ///
  /// # Returns
  /// * `Value::DistinguishedName`
  pub(crate) fn distinguished_name<T>(distinguished_name: T) -> Self
  where
    T: Into<String>,
  {
    Self::DistinguishedName(distinguished_name.into())
  }

  /// Create `Value` representing an empty value.
  ///
  /// # Returns
  /// * `Value::Empty`
  pub(crate) fn empty() -> Self {
    Self::Empty
  }

  /// Create `Value` representing an error.
  ///
  /// # Returns
  /// * `Value::Error` - Represents an error message.
  pub(crate) fn error<T>(value: T) -> Self
  where
    T: ToString,
  {
    Self::Error(value.to_string())
  }

  /// Create `Value` representing a hidden value.
  ///
  /// # Returns
  /// * `Value::Hide`
  pub(crate) fn hide() -> Self {
    Self::Hide
  }

  /// Create `Value` representing a value that should be ignored.
  ///
  /// # Returns
  /// * `Value::Ignore` - Represents an ignore message.
  pub(crate) fn ignore<T>(value: T) -> Self
  where
    T: ToString,
  {
    Self::Ignore(value.to_string())
  }

  /// Create `Value` representing a value that is not applicable.
  ///
  /// # Returns
  /// * `Value::NotApplicable`
  pub(crate) fn not_applicable() -> Self {
    Self::NotApplicable
  }

  /// Create `Value` representing a value that might be correct or not.
  ///
  /// # Parameters
  /// * `value` - `Result<T, _>` that represents the value that might be incorrect.
  /// * `default` - Value that will be shown when `value` is an `Err`.
  ///
  /// # Returns
  /// * `Value::Plain(value)` - When `value` is `Ok`.
  /// * `Value::Plain(default)` - When `value` is an `Err`.
  pub(crate) fn ok_or<T, U, E>(value: Result<T, E>, default: U) -> Self
  where
    T: ToString,
    U: ToString,
  {
    match value {
      Ok(v) => Self::plain(v.to_string()),
      Err(_) => Self::plain(default.to_string()),
    }
  }

  /// Create `Value` representing a value that might be correct or not.
  ///
  /// # Parameters
  /// * `value` - `Result<T, _>` that represents the value that might be incorrect.
  ///
  /// # Returns
  /// * `Value::Plain(value)` - When `value` is `Ok`.
  /// * `Value::Empty` - When `value` is an `Err`.
  pub(crate) fn ok_or_empty<T, E>(value: Result<T, E>) -> Self
  where
    T: ToString,
  {
    match value {
      Ok(v) => Self::plain(v.to_string()),
      Err(_) => Self::empty(),
    }
  }

  /// Create `Value` representing a value that might be correct or not.
  ///
  /// # Parameters
  /// * `value` - `Result<T, _>` that represents the value that might be incorrect.
  ///
  /// # Returns
  /// * `Value::Plain(value)` - When `value` is `Ok`.
  /// * `Value::Hide` - When `value` is an `Err`.
  pub(crate) fn ok_or_hide<T, E>(value: Result<T, E>) -> Self
  where
    T: ToString,
  {
    match value {
      Ok(v) => Self::plain(v.to_string()),
      Err(_) => Self::hide(),
    }
  }

  /// Create `Value` representing a plain value.
  ///
  /// # Parameters
  /// * `value` - Value.
  ///
  /// # Returns
  /// * `Value::Plain` - Represents the value.
  pub(crate) fn plain<T>(value: T) -> Self
  where
    T: ToString,
  {
    Self::Plain(value.to_string())
  }

  /// Create `Value` representing a result value.
  ///
  /// # Parameters
  /// * `value` - `Result<T, E>` that represents the value that might be incorrect.
  ///
  /// # Returns
  /// * `Value::Plain(value)` - When `value` is `Ok`.
  /// * `Value::Error(message)` - When `value` is an `Err`.
  pub(crate) fn result<T, E>(value: Result<T, E>) -> Self
  where
    T: ToString,
    E: ToString,
  {
    match value {
      Ok(v) => Self::plain(v.to_string()),
      Err(e) => Self::error(e.to_string()),
    }
  }

  /// Create `Value` representing a secret.
  ///
  /// # Returns
  /// * `Value::Secret` - Represents the secret.
  pub(crate) fn secret() -> Self {
    Self::Secret
  }

  /// Create `Value` representing an optional value with default.
  ///
  /// # Parameters
  /// * `value` - `Option<T>` that represents the optional value.
  /// * `default` - Value that will be shown when `value` is `None`.
  ///
  /// # Returns
  /// * `Value::Plain(value)` - When `value` is `Some`.
  /// * `Value::Plain(default)` - When `value` is `None`.
  pub(crate) fn some_or<T, U>(value: Option<T>, default: U) -> Self
  where
    T: ToString,
    U: ToString,
  {
    match value {
      Some(v) => Self::plain(v.to_string()),
      None => Self::plain(default.to_string()),
    }
  }

  /// Create `Value` representing an optional value.
  ///
  /// # Parameters
  /// * `value` - `Option<T>` that represents the optional value.
  ///
  /// # Returns
  /// * `Value::Plain(value)` - When `value` is present.
  /// * `Value::Empty` - When `value` is `None`.
  pub(crate) fn some_or_empty<T>(value: Option<T>) -> Self
  where
    T: ToString,
  {
    match value {
      Some(v) => Self::plain(v.to_string()),
      None => Self::empty(),
    }
  }

  /// Create `Value` representing a value or hide.
  ///
  /// # Parameters
  /// * `value` - `Option<T>` that represents the optional value.
  ///
  /// # Returns
  /// * `Value::Plain(value)` - When `value` is `Some`.
  /// * `Value::Hide` - When `value` is `None`.
  pub(crate) fn some_or_hide<T>(value: Option<T>) -> Self
  where
    T: ToString,
  {
    match value {
      Some(v) => Self::plain(v.to_string()),
      None => Self::hide(),
    }
  }

  /// Create `Value` representing a target.
  ///
  /// # Parameters
  /// * `value` - Target value which identifies something.
  ///
  /// # Returns
  /// * `Value::Target` - Represents the target value.
  pub(crate) fn target<T>(value: T) -> Self
  where
    T: ToString,
  {
    Self::Target(value.to_string())
  }

  /// Create `Value` representing a timestamp.
  ///
  /// # Parameters
  /// * `timestamp` - Timestamp in seconds since epoch.
  ///
  /// # Returns
  /// * `Value::Plain` - Formatted timestamp.
  pub(crate) fn timestamp_seconds(timestamp: i64) -> Self {
    match DateTime::from_timestamp_secs(timestamp) {
      Some(datetime) => Self::plain(datetime),
      None => plain!("{} (error)", timestamp),
    }
  }

  /// Create `Value` representing timestamp with expiration check.
  ///
  /// # Parameters
  /// * `timestamp` - Timestamp in seconds since epoch.
  /// * `days` - Optional number of days in the future for the warning check.
  ///
  /// # Returns
  /// * `Value::Error` - When `timestamp` is in the past.
  /// * `Value::Warn` - When `date` is present and `timestamp` will expire within `days` days.
  /// * `Value::Plain` - Otherwise.
  #[allow(dead_code)]
  pub(crate) fn timestamp_seconds_expired(timestamp: i64, days: Option<u64>) -> Self {
    match DateTime::from_timestamp_secs(timestamp) {
      Some(ref datetime) => Self::datetime_expired(datetime, days),
      None => plain!("{} (unchecked)", timestamp),
    }
  }

  /// Create `Value` representing timestamp with not-before check.
  ///
  /// # Parameters
  /// * `timestamp` - Timestamp in seconds since epoch.
  ///
  /// # Returns
  /// * `Value::Plain` - When `timestamp` is in the past.
  /// * `Value::Warn` - When `timestamp` is now or in the future.
  pub(crate) fn timestamp_seconds_not_before(timestamp: i64) -> Self {
    match DateTime::from_timestamp_secs(timestamp) {
      Some(ref datetime) => Self::datetime_not_before(datetime),
      None => plain!("{} (unchecked)", timestamp),
    }
  }

  /// Create `Value` representing an unimplemented state.
  ///
  /// # Returns
  /// * `Value::Todo`
  pub(crate) fn todo() -> Self {
    Self::Todo
  }

  /// Create `Value` representing a program flow error.
  ///
  /// # Returns
  /// * `Value::Unreachable`
  pub(crate) fn unreachable() -> Self {
    Self::Unreachable
  }

  /// Create `Value` representing a warning.
  ///
  /// # Returns
  /// * `Value::Warn` - Represents a warning message.
  pub(crate) fn warn<T>(value: T) -> Self
  where
    T: ToString,
  {
    Self::Warn(value.to_string())
  }

  const REDACTED_SECRET: &'static str = "[redacted]";

  pub(crate) fn to_decorated_string(&self, context: &Context) -> String {
    match self {
      Self::DistinguishedName(value) => context.apply_stdout_style(format_distinguished_name(value)),
      Self::Empty => "".to_string(),
      Self::Error(value) => context.apply_error_style(value),
      Self::Hide => "".to_string(),
      Self::Ignore(value) => context.apply_ignore_style(value),
      Self::NotApplicable => context.apply_ignore_style("n.a."),
      Self::Plain(value) => context.apply_stdout_style(value),
      Self::Secret => Self::REDACTED_SECRET.to_string(),
      Self::Target(value) => context.apply_target_style(value),
      Self::Todo => context.apply_error_style("TODO"),
      Self::Unreachable => context.apply_error_style("unreachable"),
      Self::Warn(value) => context.apply_warning_style(value),
    }
  }

  pub(crate) fn to_undecorated_string(&self) -> String {
    match self {
      Self::DistinguishedName(value) => format_distinguished_name(value),
      Self::Empty => "".to_string(),
      Self::Error(value) => value.to_string(),
      Self::Hide => "".to_string(),
      Self::Ignore(value) => value.to_string(),
      Self::NotApplicable => "".to_string(),
      Self::Plain(value) => value.to_string(),
      Self::Secret => Self::REDACTED_SECRET.to_string(),
      Self::Target(value) => value.to_string(),
      Self::Todo => "TODO".to_string(),
      Self::Unreachable => "unreachable".to_string(),
      Self::Warn(value) => value.to_string(),
    }
  }
}

impl Default for Value {
  fn default() -> Self {
    Self::Empty
  }
}
