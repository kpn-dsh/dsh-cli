use crate::error;
use crate::error::DshCliError;
use dsh_api::types::Notification;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

pub(crate) mod ids_formatter;
pub(crate) mod list_formatter;
pub(crate) mod unit_formatter;

/// # Defines behavior of labels
///
/// Adds capabilities to a type that defines its behavior as a label.
/// This `trait` is typically implemented on `enum` types.
pub(crate) trait Label: Eq + Hash + PartialEq + Serialize {
  /// Returns the default label text
  fn as_str(&self) -> &str;

  /// # Returns label text for use in csv
  ///
  /// Returns the text for this label when it is used in csv output.
  /// The default implementation returns the default label text.
  fn as_str_for_csv(&self) -> &str {
    self.as_str()
  }

  /// # Returns label text for use in a list
  ///
  /// Returns the text for this label when it is used in list output.
  /// The default implementation returns the default label text.
  fn as_str_for_list(&self) -> &str {
    self.as_str()
  }

  /// # Returns label text for use in unit
  ///
  /// Returns the text for this label when it is used in unit output.
  /// The default implementation returns the default label text.
  fn as_str_for_unit(&self) -> &str {
    self.as_str()
  }

  /// # Indicates whether the label is the target label
  ///
  /// Indicates whether `self` is the unique target label for this label type.
  /// The target label is the label for the field that uniquely identifies
  /// a record or data structure.
  /// Only one value of the `enum` for which the `trait` is implemented must return `true.
  /// If a target label does not make sense or is undefined for the label type,
  /// `false` must be returned.
  ///
  /// # Returns
  /// * `false` - target label does not make sense or is not defined for this `Label` type
  /// * `true` - if `self` is the target label for this label type.
  ///   Only one value can return `true`.
  fn is_target_label(&self) -> bool;
}

/// # Defines how a data type will be formatted
///
/// By implementing the `SubjectFormatter` trait for an arbitrary type you can define
/// the relation between the type and the labels that are used to designate the type's fields.
pub(crate) trait SubjectFormatter<L>
where
  L: Label,
{
  /// # Returns the value for a label
  ///
  /// This method must return the value of the type's field that corresponds
  /// to the provided `label`.
  /// If the `label` is the target label for the data type, the `target_id` parameter
  /// must be returned. Else the returned value will be picked from the data structure,
  /// depending on the `label` value.
  fn value(&self, label: &L, target_id: &str) -> String;

  /// # Returns the target id
  ///
  /// If a unique identifying target id exists for the data type, this method must return its value
  /// wrapped in a `Some`. Else it should return `None`.
  fn target_id(&self) -> Option<String> {
    None
  }
}

/// # Defines how a `String` pair can be formatted
///
/// The first value in the tuple (`self.0`) is handled as the target id,
/// the second value in the tuple (`self.1`) as the only value.
impl<L> SubjectFormatter<L> for (String, String)
where
  L: Label,
{
  fn value(&self, label: &L, _target_id: &str) -> String {
    if label.is_target_label() {
      self.0.clone()
    } else {
      self.1.clone()
    }
  }

  fn target_id(&self) -> Option<String> {
    Some(self.0.clone())
  }
}

/// # Defines how a `String` can be formatted
///
/// The value of the `String` is handled as the target id and the value.
/// There is no target label defined for this case.
/// The `Label` trait is implemented for `String` and can therefor be used
/// in combination with this implementation.
impl<L> SubjectFormatter<L> for String
where
  L: Label,
{
  fn value(&self, label: &L, target_id: &str) -> String {
    if label.is_target_label() {
      target_id.to_string()
    } else {
      self.clone()
    }
  }

  fn target_id(&self) -> Option<String> {
    Some(self.clone())
  }
}

/// # Defines how a `HashMap` can be formatted
///
/// The key of the `HashMap` must implement `Label`.
/// This implementation does not allow the specification of a target id.
impl<L> SubjectFormatter<L> for HashMap<L, String>
where
  L: Label,
{
  fn value(&self, label: &L, target_id: &str) -> String {
    if label.is_target_label() {
      target_id.to_string()
    } else {
      self.get(label).map(|s| s.to_string()).unwrap_or_default()
    }
  }

  fn target_id(&self) -> Option<String> {
    None
  }
}

/// # Makes `String` available as a label
impl Label for String {
  fn as_str(&self) -> &str {
    self.as_str()
  }

  fn is_target_label(&self) -> bool {
    false
  }
}

#[derive(clap::ValueEnum, Eq, Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum OutputFormat {
  /// Output will be formatted as comma separated values
  #[serde(rename = "csv")]
  Csv,
  /// Output will be in json format
  #[serde(rename = "json")]
  Json,
  /// Output will be in compact json format
  #[serde(rename = "json-compact")]
  JsonCompact,
  /// Output will be formatted as plain text
  #[serde(rename = "plain")]
  Plain,
  /// No output will be generated
  #[serde(rename = "quiet")]
  Quiet,
  /// Output will be formatted as a table with borders
  #[serde(rename = "table")]
  Table,
  /// Output will be formatted as a table without borders
  #[serde(rename = "table-no-border")]
  TableNoBorder,
  /// Output will be in toml format
  #[serde(rename = "toml")]
  Toml,
  /// Output will be in compact toml format
  #[serde(rename = "toml-compact")]
  TomlCompact,
  /// Output will be in yaml format
  #[serde(rename = "yaml")]
  Yaml,
}

impl Display for OutputFormat {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      OutputFormat::Csv => write!(f, "csv"),
      OutputFormat::Json => write!(f, "json"),
      OutputFormat::JsonCompact => write!(f, "json-compact"),
      OutputFormat::Plain => write!(f, "plain"),
      OutputFormat::Quiet => write!(f, "quiet"),
      OutputFormat::Table => write!(f, "table"),
      OutputFormat::TableNoBorder => write!(f, "table-no-border"),
      OutputFormat::Toml => write!(f, "toml"),
      OutputFormat::TomlCompact => write!(f, "toml-compact"),
      OutputFormat::Yaml => write!(f, "yaml"),
    }
  }
}

impl TryFrom<&str> for OutputFormat {
  type Error = DshCliError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "csv" => Ok(Self::Csv),
      "json" => Ok(Self::Json),
      "json-compact" => Ok(Self::JsonCompact),
      "plain" => Ok(Self::Plain),
      "quiet" => Ok(Self::Quiet),
      "table" => Ok(Self::Table),
      "table-no-border" => Ok(Self::TableNoBorder),
      "toml" => Ok(Self::Toml),
      "toml-compact" => Ok(Self::TomlCompact),
      "yaml" => Ok(Self::Yaml),
      _ => Err(error!("invalid output format '{}'", value)),
    }
  }
}

/// Converts hashmap to a sorted table string
pub(crate) fn hashmap_to_table<K: AsRef<str>, V: AsRef<str>>(hashmap: &HashMap<K, V>) -> String {
  let mut key_value_length_pairs: Vec<(&str, Vec<&str>, usize)> = hashmap
    .iter()
    .map(|(key, value)| (key.as_ref(), value.as_ref().split("\n").collect_vec(), key.as_ref().len()))
    .collect_vec();
  match key_value_length_pairs.iter().map(|(_, _, len)| len).max().cloned() {
    Some(first_column_width) => {
      key_value_length_pairs.sort_by(|(key_a, _, _), (key_b, _, _)| key_a.cmp(key_b));
      key_value_length_pairs
        .into_iter()
        .map(|(key, values, key_length)| {
          let mut lines = vec![];
          let mut values_iter = values.iter();
          if let Some(first) = values_iter.next() {
            lines.push(format!("{}{}  {}", key, " ".repeat(first_column_width - key_length), first));
          }
          for rest in values_iter {
            lines.push(format!("{}  {}", " ".repeat(first_column_width), rest));
          }
          lines.join("\n")
        })
        .collect_vec()
        .join("\n")
    }
    None => "".to_string(),
  }
}

/// Converts hashmap to a sorted vector of strings
pub(crate) fn hashmap_to_vec<K: AsRef<str>, V: AsRef<str>>(hashmap: &HashMap<K, V>) -> Vec<String> {
  let mut key_value_length_pairs: Vec<(&str, Vec<&str>, usize)> = hashmap
    .iter()
    .map(|(key, value)| (key.as_ref(), value.as_ref().split("\n").collect_vec(), key.as_ref().len()))
    .collect_vec();
  match key_value_length_pairs.iter().map(|(_, _, len)| len).max().cloned() {
    Some(first_column_width) => {
      key_value_length_pairs.sort_by(|(key_a, _, _), (key_b, _, _)| key_a.cmp(key_b));
      key_value_length_pairs
        .into_iter()
        .map(|(key, values, key_length)| {
          let mut lines = vec![];
          let mut values_iter = values.iter();
          if let Some(first) = values_iter.next() {
            lines.push(format!("{}{}  {}", key, " ".repeat(first_column_width - key_length), first));
          }
          for rest in values_iter {
            lines.push(format!("{}  {}", " ".repeat(first_column_width), rest));
          }
          lines.join("\n")
        })
        .collect_vec()
    }
    None => vec![],
  }
}

/// Converts vector of vectors to a table string
pub(crate) fn vec_to_table<K: AsRef<str>, V: AsRef<str>>(rows: &[(K, Vec<V>)]) -> String {
  let key_values_length_pairs: Vec<(&str, Vec<&str>, usize)> = rows
    .iter()
    .map(|(key, values)| (key.as_ref(), values.iter().map(|value| value.as_ref()).collect_vec(), key.as_ref().len()))
    .collect_vec();
  match key_values_length_pairs.iter().map(|(_, _, len)| len).max().cloned() {
    Some(first_column_width) => key_values_length_pairs
      .into_iter()
      .map(|(key, values, key_length)| {
        let mut lines = vec![];
        let mut values_iter = values.iter();
        if let Some(first) = values_iter.next() {
          lines.push(format!("{}{}  {}", key, " ".repeat(first_column_width - key_length), first));
        }
        for rest in values_iter {
          lines.push(format!("{}  {}", " ".repeat(first_column_width), rest));
        }
        lines.join("\n")
      })
      .collect_vec()
      .join("\n"),
    None => "".to_string(),
  }
}

pub(crate) fn notifications_to_string(notifications: &[Notification]) -> String {
  notifications.iter().map(notification_to_string).collect_vec().join(", ")
}

pub(crate) fn notification_to_string(notification: &Notification) -> String {
  format!(
    "{}\n{}\n{}",
    if notification.remove { "remove".to_string() } else { "create/update".to_string() },
    notification.message,
    notification.args.iter().map(|(key, value)| format!("{}:{}", key, value)).collect_vec().join("\n"),
  )
}
