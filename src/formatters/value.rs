use crate::context::Context;
use chrono::LocalResult::{Ambiguous, Single};
use chrono::{DateTime, Days, Local, TimeZone, Utc};

#[derive(Clone, Debug)]
pub(crate) enum Value {
  Empty,
  Error(String),
  Ignore(String),
  NotApplicable,
  Plain(String),
  Secret,
  Target(String),
  Todo,
  Warn(String),
}

impl Value {
  /// Create `Value` representing date/time
  ///
  /// # Parameters
  /// * `date_time` - References a `DateTime<Utc>` struct representing the date/time.
  ///
  /// # Returns
  /// * `Value::Plain`
  #[allow(dead_code)]
  pub(crate) fn date_time(date_time: &DateTime<Utc>) -> Self {
    Self::plain(date_time)
  }

  /// Create `Value` representing date/time with expiration check
  ///
  /// # Parameters
  /// * `date_time` - References a `DateTime<Utc>` struct representing the date/time.
  /// * `days` - Optional number of days in the future for the warning check.
  ///
  /// # Returns
  /// * `Value::Error` - When `date_time` is in the past.
  /// * `Value::Warn` - When `date` is present and `date_time` will expire within `days` days.
  /// * `Value::Plain` - Otherwise.
  pub(crate) fn date_time_expired(date_time: &DateTime<Utc>, days: Option<u64>) -> Self {
    if date_time < &Local::now() {
      Self::error(date_time)
    } else {
      match days {
        Some(warning_days) => match Local::now().checked_add_days(Days::new(warning_days)) {
          Some(expiration_warning_date) => {
            if date_time < &expiration_warning_date {
              Self::warn(date_time)
            } else {
              Self::plain(date_time)
            }
          }
          None => Self::plain(format!("{} (unchecked)", date_time)),
        },
        None => Self::plain(date_time),
      }
    }
  }

  /// Create `Value` representing date/time with not-before check
  ///
  /// # Parameters
  /// * `date_time` - References a `DateTime<Utc>` struct representing the date/time.
  ///
  /// # Returns
  /// * `Value::Plain` - When `date_time` is in the past.
  /// * `Value::Warn` - When `date_time` is now or in the future.
  pub(crate) fn date_time_not_before(date_time: &DateTime<Utc>) -> Self {
    if date_time > &Local::now() {
      Self::warn(date_time.to_string())
    } else {
      Self::plain(date_time.to_string())
    }
  }

  /// Create `Value` representing an empty value
  ///
  /// # Returns
  /// * `Value::Empty`
  pub(crate) fn empty() -> Self {
    Self::Empty
  }

  /// Create `Value` representing an error
  ///
  /// # Returns
  /// * `Value::Error` - Represents an error message.
  pub(crate) fn error<T>(value: T) -> Self
  where
    T: ToString,
  {
    Self::Error(value.to_string())
  }

  /// Create `Value` representing a value that should be ignored
  ///
  /// # Returns
  /// * `Value::Ignore` - Represents an ignore message.
  pub(crate) fn ignore<T>(value: T) -> Self
  where
    T: ToString,
  {
    Self::Ignore(value.to_string())
  }

  /// Create `Value` representing a value that is not applicable
  ///
  /// # Returns
  /// * `Value::NotApplicable`
  pub(crate) fn not_applicable() -> Self {
    Self::NotApplicable
  }

  /// Create `Value` representing a value that might be correct or not
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

  /// Create `Value` representing an optional value
  ///
  /// # Parameters
  /// * `value` - `Option<T>` that represents the optional value.
  ///
  /// # Returns
  /// * `Value::Plain(value)` - When `value` is present.
  /// * `Value::Empty` - When `value` is `None`.
  pub(crate) fn option<T>(value: Option<T>) -> Self
  where
    T: ToString,
  {
    match value {
      Some(v) => Self::plain(v.to_string()),
      None => Self::empty(),
    }
  }

  /// Create `Value` representing a plain value
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

  /// Create `Value` representing a secret
  ///
  /// # Returns
  /// * `Value::Secret` - Represents the secret.
  pub(crate) fn secret() -> Self {
    Self::Secret
  }

  /// Create `Value` representing an optional value with default
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

  /// Create `Value` representing a target
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

  /// Create `Value` representing a timestamp
  ///
  /// # Parameters
  /// * `timestamp` - Timestamp in seconds since epoch.
  ///
  /// # Returns
  /// * `Value::Plain` - Formatted timestamp.
  pub(crate) fn timestamp_seconds(timestamp: i64) -> Self {
    match Utc.timestamp_opt(timestamp, 0) {
      Single(ref single) => Self::plain(single),
      Ambiguous(ref ambiguous, _) => Self::plain(ambiguous),
      _ => Self::plain(timestamp),
    }
  }

  /// Create `Value` representing timestamp with expiration check
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
    match Utc.timestamp_opt(timestamp, 0) {
      Single(ref single) => Self::date_time_expired(single, days),
      Ambiguous(ref ambiguous, _) => Self::date_time_expired(ambiguous, days),
      _ => Self::plain(format!("{} (unchecked)", timestamp)),
    }
  }

  /// Create `Value` representing timestamp with not-before check
  ///
  /// # Parameters
  /// * `timestamp` - Timestamp in seconds since epoch.
  ///
  /// # Returns
  /// * `Value::Plain` - When `timestamp` is in the past.
  /// * `Value::Warn` - When `timestamp` is now or in the future.
  pub(crate) fn timestamp_seconds_not_before(timestamp: i64) -> Self {
    match Utc.timestamp_opt(timestamp, 0) {
      Single(ref single) => Self::date_time_not_before(single),
      Ambiguous(ref ambiguous, _) => Self::date_time_not_before(ambiguous),
      _ => Self::plain(format!("{} (unchecked)", timestamp)),
    }
  }

  /// Create `Value` representing a value that is not yet implemented
  ///
  /// # Returns
  /// * `Value::Todo`
  pub(crate) fn todo() -> Self {
    Self::Todo
  }

  /// Create `Value` representing a warning
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
      Self::Empty => "".to_string(),
      Self::Error(value) => context.apply_error_style(value),
      Self::Ignore(value) => context.apply_ignore_style(value),
      Self::NotApplicable => context.apply_ignore_style("n.a."),
      Self::Plain(value) => context.apply_stdout_style(value),
      Self::Secret => Self::REDACTED_SECRET.to_string(),
      Self::Target(value) => context.apply_target_style(value),
      Self::Todo => context.apply_warning_style("todo"),
      Self::Warn(value) => context.apply_warning_style(value),
    }
  }

  pub(crate) fn to_undecorated_string(&self) -> String {
    match self {
      Self::Empty => "".to_string(),
      Self::Error(value) => value.to_string(),
      Self::Ignore(value) => value.to_string(),
      Self::NotApplicable => "".to_string(),
      Self::Plain(value) => value.to_string(),
      Self::Secret => Self::REDACTED_SECRET.to_string(),
      Self::Target(value) => value.to_string(),
      Self::Todo => "todo".to_string(),
      Self::Warn(value) => value.to_string(),
    }
  }
}

impl Default for Value {
  fn default() -> Self {
    Self::Empty
  }
}
