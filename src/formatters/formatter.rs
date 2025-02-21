use serde::Serialize;
use std::collections::HashMap;
use std::hash::Hash;

/// # Defines behavior of labels
///
/// Adds capabilities to a type that defines its behavior as a label.
/// This `trait` is typically implemented on `enum` types.
pub trait Label: Eq + Hash + PartialEq + Serialize {
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
pub trait SubjectFormatter<L>
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

  /// # Returns the target label for the data type
  ///
  /// If the data type has a unique identifying target id, this method must return its label,
  /// wrapped in a `Some`.
  /// Else it should return `None`.
  #[allow(dead_code)]
  fn target_label(&self) -> Option<L>;
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

  fn target_label(&self) -> Option<L> {
    None
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

  fn target_label(&self) -> Option<L> {
    None
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

  fn target_label(&self) -> Option<L> {
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

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum PropertyLabel {
  Property,
  Value,
}

impl Label for PropertyLabel {
  fn as_str(&self) -> &str {
    match self {
      PropertyLabel::Property => "property",
      PropertyLabel::Value => "value",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Property)
  }
}

pub static PROPERTY_LABELS: [PropertyLabel; 2] = [PropertyLabel::Property, PropertyLabel::Value];

#[derive(Eq, Hash, PartialEq, Serialize)]
pub(crate) enum EnvironmentVariableLabel {
  Variable,
  Value,
}

impl Label for EnvironmentVariableLabel {
  fn as_str(&self) -> &str {
    match self {
      EnvironmentVariableLabel::Variable => "environment variable",
      EnvironmentVariableLabel::Value => "value",
    }
  }

  fn is_target_label(&self) -> bool {
    matches!(self, Self::Variable)
  }
}

pub static ENVIRONMENT_VARIABLE_LABELS: [EnvironmentVariableLabel; 2] = [EnvironmentVariableLabel::Variable, EnvironmentVariableLabel::Value];

/// Converts hashmap to a sorted table string
pub fn hashmap_to_table<K: AsRef<str>, V: AsRef<str>>(hashmap: &HashMap<K, V>) -> String {
  let mut key_value_length_pairs: Vec<(&str, &str, usize)> = hashmap
    .iter()
    .map(|(key, value)| (key.as_ref(), value.as_ref(), key.as_ref().len()))
    .collect::<Vec<_>>();
  match key_value_length_pairs.iter().map(|(_, _, len)| len).max().cloned() {
    Some(first_column_width) => {
      key_value_length_pairs.sort_by(|(key_a, _, _), (key_b, _, _)| key_a.cmp(key_b));
      key_value_length_pairs
        .into_iter()
        .map(|(key, value, len)| format!("{}{}  {}", key, " ".repeat(first_column_width - len), value))
        .collect::<Vec<_>>()
        .join("\n")
    }
    None => "".to_string(),
  }
}
