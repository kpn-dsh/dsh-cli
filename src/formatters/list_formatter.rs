use crate::context::Context;
use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::OutputFormat;
use itertools::Itertools;
use serde::Serialize;
use std::marker::PhantomData;
use tabled::settings::peaker::PriorityMax;
use tabled::settings::{Padding, Width};
use tabled::{builder::Builder as TabledBuilder, settings::Style};

pub struct ListFormatter<'a, L: Label, V: SubjectFormatter<L>> {
  labels: &'a [L],
  target_label: Option<&'a str>,
  values: Vec<(String, &'a V)>,
  context: &'a Context,
  phantom: PhantomData<&'a V>,
}

impl<'a, L, V> ListFormatter<'a, L, V>
where
  L: Label,
  V: SubjectFormatter<L> + Serialize,
{
  /// # Creates a new `ListFormatter`
  pub fn new(labels: &'a [L], target_label: Option<&'a str>, context: &'a Context) -> Self {
    Self { labels, target_label, values: vec![], context, phantom: PhantomData }
  }

  /// # Pushes target ids and values
  ///
  /// This method expects two slices containing target ids and corresponding values.
  /// It is assumed that both slices contain the same number of values
  /// and that target ids and values with the same index belong to each other.
  pub fn push_target_ids_and_values(&mut self, target_ids: &[String], values: &'a [V]) -> &Self {
    for (target_id, value) in target_ids.iter().zip(values).collect::<Vec<_>>() {
      self.values.push((target_id.clone(), value));
    }
    self
  }

  /// # Pushes target id/value pairs
  ///
  /// This method expects a slice containing target id/value pairs.
  pub fn push_target_id_value_pairs(&mut self, values: &'a [(String, V)]) -> &Self {
    for (target_id, value) in values {
      self.values.push((target_id.clone(), value));
    }
    self
  }

  pub fn _push_value(&mut self, value: &'a V) -> &Self {
    match value.target_id() {
      Some(target_id) => self.values.push((target_id, value)),
      None => self.values.push(("".to_string(), value)),
    }
    self
  }

  pub fn push_values(&mut self, values: &'a [V]) -> &Self {
    for value in values {
      match value.target_id() {
        Some(target_id) => self.values.push((target_id, value)),
        None => self.values.push(("".to_string(), value)),
      }
    }
    self
  }

  pub fn push_target_id_value(&mut self, target_id: String, value: &'a V) -> &Self {
    self.values.push((target_id, value));
    self
  }

  pub fn is_empty(&self) -> bool {
    self.values.is_empty()
  }

  fn label_to_header(&self, label: &L) -> String {
    if label.is_target_label() {
      self.add_csv_quote(self.target_label.unwrap_or(label.as_str_for_list()))
    } else {
      self.add_csv_quote(label.as_str_for_list())
    }
  }

  fn add_csv_quote(&self, value: &str) -> String {
    match self.context.csv_quote {
      Some(csv_quote) => format!("{}{}{}", csv_quote, value, csv_quote),
      None => value.to_string(),
    }
  }

  pub fn print(&self) -> Result<(), String> {
    match self.context.output_format {
      OutputFormat::Csv => {
        if self.context.show_headers {
          self.context.print(
            self
              .labels
              .iter()
              .map(|label| self.label_to_header(label))
              .join(self.context.csv_separator.as_str()),
          );
        }
        for (target_id, value) in &self.values {
          self.context.print(
            self
              .labels
              .iter()
              .map(|label| self.add_csv_quote(value.value(label, target_id).as_str()))
              .join(self.context.csv_separator.as_str()),
          );
        }
        Ok(())
      }

      OutputFormat::Json => match self.simplified_values() {
        Some(simplified_values) => match serde_json::to_string_pretty(&simplified_values) {
          Ok(json) => {
            self.context.print(json);
            Ok(())
          }
          Err(error) => Err(format!("could not convert simplified values to json ({})", error)),
        },
        None => match serde_json::to_string_pretty(&self.values) {
          Ok(json) => {
            self.context.print(json);
            Ok(())
          }
          Err(error) => Err(format!("could not convert values to json ({})", error)),
        },
      },

      OutputFormat::JsonCompact => match self.simplified_values() {
        Some(simplified_values) => match serde_json::to_string(&simplified_values) {
          Ok(json) => {
            self.context.print(json);
            Ok(())
          }
          Err(error) => Err(format!("could not convert simplified values to json compact ({})", error)),
        },
        None => match serde_json::to_string_pretty(&self.values) {
          Ok(json) => {
            self.context.print(json);
            Ok(())
          }
          Err(error) => Err(format!("could not convert values to json compact ({})", error)),
        },
      },

      OutputFormat::Plain => {
        if self.context.show_headers {
          self.context.print(self.labels.iter().map(|label| label.as_str()).join(","));
        }
        for (target_id, value) in &self.values {
          self.context.print(self.labels.iter().map(|label| value.value(label, target_id)).join(","));
        }
        Ok(())
      }

      OutputFormat::Quiet => Ok(()),

      OutputFormat::Table => {
        let mut tabled_builder = TabledBuilder::default();
        tabled_builder.push_record(
          self
            .labels
            .iter()
            .map(|label| if label.is_target_label() { self.target_label.unwrap_or(label.as_str_for_list()) } else { label.as_str_for_list() }),
        );
        let mut last_target_id: Option<String> = None;
        for (target_id, value) in &self.values {
          let record = self.labels.iter().map(|label| {
            if label.is_target_label() {
              if last_target_id.clone().is_some_and(|ref last| last == target_id) {
                "".to_string()
              } else {
                last_target_id = Some(target_id.to_string());
                value.value(label, target_id)
              }
            } else {
              value.value(label, target_id)
            }
          });
          tabled_builder.push_record(record);
        }
        let mut table = tabled_builder.build();
        if let Some(terminal_width) = self.context.terminal_width {
          table.with(Width::truncate(terminal_width).priority(PriorityMax::new(true)).suffix("..."));
        }
        table.with(Padding::new(1, 1, 0, 0));
        if self.values.is_empty() {
          table.with(Style::modern());
        } else {
          table.with(Style::sharp());
        }
        self.context.print(table.to_string());
        Ok(())
      }

      OutputFormat::TableNoBorder => {
        let mut tabled_builder = TabledBuilder::default();
        tabled_builder.push_record(
          self
            .labels
            .iter()
            .map(|label| if label.is_target_label() { self.target_label.unwrap_or(label.as_str_for_list()) } else { label.as_str_for_list() }),
        );
        for (target_id, value) in &self.values {
          tabled_builder.push_record(self.labels.iter().map(|label| value.value(label, target_id)));
        }
        let mut table = tabled_builder.build();
        if let Some(terminal_width) = self.context.terminal_width {
          table.with(Width::truncate(terminal_width).priority(PriorityMax::new(true)).suffix("..."));
        }
        table.with(Padding::new(0, 2, 0, 0));
        table.with(Style::empty());
        self.context.print(table.to_string());
        Ok(())
      }

      OutputFormat::Toml => match self.simplified_values() {
        Some(simplified_values) => match toml::to_string_pretty(&simplified_values) {
          Ok(json) => {
            self.context.print(json);
            Ok(())
          }
          Err(error) => Err(format!("could not convert simplified values to toml ({})", error)),
        },
        None => match serde_json::to_string_pretty(&self.values) {
          Ok(json) => {
            self.context.print(json);
            Ok(())
          }
          Err(error) => Err(format!("could not convert values to toml ({})", error)),
        },
      },

      OutputFormat::TomlCompact => match self.simplified_values() {
        Some(simplified_values) => match toml::to_string(&simplified_values) {
          Ok(json) => {
            self.context.print(json);
            Ok(())
          }
          Err(error) => Err(format!("could not convert simplified values to toml compact ({})", error)),
        },
        None => match serde_json::to_string_pretty(&self.values) {
          Ok(json) => {
            self.context.print(json);
            Ok(())
          }
          Err(error) => Err(format!("could not convert values to toml compact ({})", error)),
        },
      },

      OutputFormat::Yaml => match self.simplified_values() {
        Some(simplified_values) => match serde_yaml::to_string(&simplified_values) {
          Ok(json) => {
            self.context.print(json);
            Ok(())
          }
          Err(error) => Err(format!("could not convert simplified values to yaml ({})", error)),
        },
        None => match serde_json::to_string_pretty(&self.values) {
          Ok(json) => {
            self.context.print(json);
            Ok(())
          }
          Err(error) => Err(format!("could not convert values to yaml ({})", error)),
        },
      },
    }
  }

  fn has_target_label(&self) -> bool {
    self.labels.iter().any(|label| label.is_target_label())
  }

  fn simplified_values(&self) -> Option<Vec<&'a V>> {
    if self.has_target_label() {
      None
    } else {
      Some(self.values.iter().map(|(_, value)| *value).collect::<Vec<_>>())
    }
  }
}
