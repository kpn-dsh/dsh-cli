use clap::builder::TypedValueParser;
use clap::error::ErrorKind;
use clap::{Arg, Command};
use std::any::type_name;
use std::ffi::OsStr;
use std::fmt::Display;
use std::str::FromStr;

/// Arguments parser for ranged values
#[derive(Clone)]
pub(crate) struct RangedValueParser<T> {
  lower_bound: Option<T>,
  upper_bound: Option<T>,
}

impl<T> RangedValueParser<T>
where
  T: Display,
{
  /// Create new `RangedValueParser`
  ///
  /// ## Parameters
  /// * `lower_bound` - lower bound, values must be greater than or equal to this value
  /// * `upper_bound` - upper bound, values must be lower than or equal to this value
  pub(crate) fn new(lower_bound: T, upper_bound: T) -> RangedValueParser<T> {
    RangedValueParser { lower_bound: Some(lower_bound), upper_bound: Some(upper_bound) }
  }

  /// Create new `RangedValueParser` with lower bound
  ///
  /// ## Parameters
  /// * `lower_bound` - lower bound, values must be greater than or equal to this value
  pub(crate) fn with_lower(lower_bound: T) -> RangedValueParser<T> {
    RangedValueParser { lower_bound: Some(lower_bound), upper_bound: None }
  }

  /// Create new `RangedValueParser` with upper bound
  ///
  /// ## Parameters
  /// * `upper_bound` - upper bound, values must be lower than or equal to this value
  pub(crate) fn _with_upper(upper_bound: T) -> RangedValueParser<T> {
    RangedValueParser { lower_bound: None, upper_bound: Some(upper_bound) }
  }

  fn long_argument(arg: Option<&Arg>) -> String {
    arg.and_then(|arg| arg.get_long()).map(|long| long.to_string()).unwrap_or("".to_string())
  }

  fn value_name(arg: Option<&Arg>) -> String {
    arg
      .and_then(|arg| arg.get_value_names().map(|value_names| value_names.join(" ")))
      .unwrap_or("".to_string())
  }

  fn parse_error_message(&self, value: &str, arg: Option<&Arg>) -> String {
    match type_name::<T>() {
      "f64" => format!(
        "invalid value '{}' for '--{} [{}]'\n  [{}] must be a floating point value",
        value,
        Self::long_argument(arg),
        Self::value_name(arg),
        Self::value_name(arg)
      ),
      "i64" => format!(
        "invalid value '{}' for '--{} [{}]'\n  [{}] must be an integer value",
        value,
        Self::long_argument(arg),
        Self::value_name(arg),
        Self::value_name(arg)
      ),
      "u32" | "u64" => format!(
        "invalid value '{}' for '--{} [{}]'\n  [{}] must be an unsigned integer value",
        value,
        Self::long_argument(arg),
        Self::value_name(arg),
        Self::value_name(arg)
      ),
      _ => unreachable!(),
    }
  }

  fn range_error_message(&self, value: T, arg: Option<&Arg>) -> String {
    match (&self.lower_bound, &self.upper_bound) {
      (None, None) => "".to_string(),
      (None, Some(upper_bound)) => format!(
        "invalid value '{}' for '--{} [{}]'\n  [{}] must be lower than or equal to {}",
        value,
        Self::long_argument(arg),
        Self::value_name(arg),
        Self::value_name(arg),
        upper_bound
      ),
      (Some(lower_bound), None) => format!(
        "invalid value '{}' for '--{} [{}]'\n  [{}] must be greater than or equal to {}",
        value,
        Self::long_argument(arg),
        Self::value_name(arg),
        Self::value_name(arg),
        lower_bound
      ),
      (Some(lower_bound), Some(upper_bound)) => format!(
        "invalid value '{}' for '--{} [{}]'\n  [{}] must be greater than or equal to {} and lower than or equal to {}",
        value,
        Self::long_argument(arg),
        Self::value_name(arg),
        Self::value_name(arg),
        lower_bound,
        upper_bound
      ),
    }
  }
}

impl<T> TypedValueParser for RangedValueParser<T>
where
  T: Clone + Display + FromStr + PartialOrd + Send + 'static + Sync,
{
  type Value = T;

  fn parse_ref(&self, _cmd: &Command, arg: Option<&Arg>, os_str: &OsStr) -> Result<Self::Value, clap::Error> {
    match os_str.to_str() {
      Some(unicode_str) => match unicode_str.parse::<T>() {
        Ok(value) => {
          if self.lower_bound.clone().is_none_or(|lower_bound| lower_bound <= value) && self.upper_bound.clone().is_none_or(|upper_bound| upper_bound >= value) {
            Ok(value)
          } else {
            Err(clap::Error::raw(ErrorKind::ValueValidation, self.range_error_message(value, arg)))
          }
        }
        Err(_) => Err(clap::Error::raw(ErrorKind::ValueValidation, self.parse_error_message(unicode_str, arg))),
      },
      None => Err(clap::Error::raw(ErrorKind::ValueValidation, "invalid unicode")),
    }
  }
}
