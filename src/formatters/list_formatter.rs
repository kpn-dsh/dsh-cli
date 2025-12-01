use crate::context::Context;
use crate::error::DshCliError;
use crate::formatters::OutputFormat;
use crate::formatters::{Label, SubjectFormatter};
use crate::{error, DshCliResult};
use itertools::Itertools;
use serde::Serialize;
use std::marker::PhantomData;
use tabled::settings::peaker::PriorityMax;
use tabled::settings::{Padding, Width};
use tabled::{builder::Builder as TabledBuilder, settings::Style};

pub(crate) struct ListFormatter<'a, L: Label, V: SubjectFormatter<L>> {
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
  pub(crate) fn new(labels: &'a [L], target_label: Option<&'a str>, context: &'a Context) -> Self {
    Self { labels, target_label, values: vec![], context, phantom: PhantomData }
  }

  /// # Pushes target ids and values
  ///
  /// This method expects two slices containing target ids and corresponding values.
  /// It is assumed that both slices contain the same number of values
  /// and that target ids and values with the same index belong to each other.
  pub(crate) fn push_target_ids_and_values(&mut self, target_ids: &[String], values: &'a [V]) -> &Self {
    for (target_id, value) in target_ids.iter().zip(values).collect_vec() {
      self.values.push((target_id.clone(), value));
    }
    self
  }

  /// # Pushes target id/value pairs
  ///
  /// This method expects a slice containing target id/value pairs.
  pub(crate) fn push_target_id_value_pairs(&mut self, values: &'a [(String, V)]) -> &Self {
    for (target_id, value) in values {
      self.values.push((target_id.clone(), value));
    }
    self
  }

  pub(crate) fn _push_value(&mut self, value: &'a V) -> &Self {
    match value.target_id() {
      Some(target_id) => self.values.push((target_id, value)),
      None => self.values.push(("".to_string(), value)),
    }
    self
  }

  pub(crate) fn push_values(&mut self, values: &'a [V]) -> &Self {
    for value in values {
      match value.target_id() {
        Some(target_id) => self.values.push((target_id, value)),
        None => self.values.push(("".to_string(), value)),
      }
    }
    self
  }

  pub(crate) fn push_target_id_value(&mut self, target_id: String, value: &'a V) -> &Self {
    self.values.push((target_id, value));
    self
  }

  pub(crate) fn is_empty(&self) -> bool {
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
    match self.context.csv_quote() {
      Some(csv_quote) => format!("{}{}{}", csv_quote, value, csv_quote),
      None => value.to_string(),
    }
  }

  pub(crate) fn print(&self, default_output_format: Option<OutputFormat>) -> DshCliResult<()> {
    match self.context.output_format(default_output_format) {
      OutputFormat::Csv => self.print_csv(),
      OutputFormat::Json => self.print_json(),
      OutputFormat::JsonCompact => self.print_json_compact(),
      OutputFormat::Plain => self.print_plain(),
      OutputFormat::Quiet => Ok(()),
      OutputFormat::Table => self.print_table(),
      OutputFormat::TableNoBorder => self.print_table_no_borders(),
      OutputFormat::Toml => self.print_toml(),
      OutputFormat::TomlCompact => self.print_toml_compact(),
      OutputFormat::Yaml => self.print_yaml(),
    }
  }

  pub(crate) fn print_non_serializable(&self, default_output_format: Option<OutputFormat>) -> DshCliResult<()> {
    match self.context.output_format(default_output_format.clone()) {
      OutputFormat::Csv => self.print_csv(),
      OutputFormat::Json => Err(error!("serialization to json is not supported for this type")),
      OutputFormat::JsonCompact => Err(error!("serialization to compact json is not supported for this type")),
      OutputFormat::Plain => Err(error!("plain unit print not yet implemented")),
      OutputFormat::Quiet => Ok(()),
      OutputFormat::Table => self.print_table(),
      OutputFormat::TableNoBorder => self.print_table_no_borders(),
      OutputFormat::Toml => Err(error!("serialization to toml is not supported for this type")),
      OutputFormat::TomlCompact => Err(error!("serialization to compact toml is not supported for this type")),
      OutputFormat::Yaml => Err(error!("serialization to yaml is not supported for this type")),
    }
  }

  fn print_csv(&self) -> DshCliResult<()> {
    if self.context.show_headers() {
      self.context.print(
        self
          .labels
          .iter()
          .map(|label| self.label_to_header(label))
          .join(self.context.csv_separator().as_str()),
      );
    }
    for (target_id, value) in &self.values {
      self.context.print(
        self
          .labels
          .iter()
          .map(|label| self.add_csv_quote(value.value(label, target_id).as_str()))
          .join(self.context.csv_separator().as_str()),
      );
    }
    Ok(())
  }

  fn print_json(&self) -> DshCliResult<()> {
    match self.simplified_values() {
      Some(simplified_values) => match serde_json::to_string_pretty(&simplified_values) {
        Ok(json) => {
          self.context.print(json);
          Ok(())
        }
        Err(error) => Err(error!("could not convert simplified values to json ({})", error)),
      },
      None => match serde_json::to_string_pretty(&self.values) {
        Ok(json) => {
          self.context.print(json);
          Ok(())
        }
        Err(error) => Err(error!("could not convert values to json ({})", error)),
      },
    }
  }

  fn print_json_compact(&self) -> DshCliResult<()> {
    match self.simplified_values() {
      Some(simplified_values) => match serde_json::to_string(&simplified_values) {
        Ok(json) => {
          self.context.print(json);
          Ok(())
        }
        Err(error) => Err(error!("could not convert simplified values to json compact ({})", error)),
      },
      None => match serde_json::to_string(&self.values) {
        Ok(json) => {
          self.context.print(json);
          Ok(())
        }
        Err(error) => Err(error!("could not convert values to json compact ({})", error)),
      },
    }
  }

  fn print_plain(&self) -> Result<(), DshCliError> {
    if self.context.show_headers() {
      self.context.print(self.labels.iter().map(|label| label.as_str()).join(","));
    }
    for (target_id, value) in &self.values {
      self.context.print(self.labels.iter().map(|label| value.value(label, target_id)).join(","));
    }
    Ok(())
  }

  fn print_table(&self) -> Result<(), DshCliError> {
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
          let target_id_value = value.value(label, target_id);
          if last_target_id.clone().is_some_and(|last| last == target_id_value) {
            "".to_string()
          } else {
            last_target_id = Some(target_id_value.clone());
            target_id_value
          }
        } else {
          value.value(label, target_id)
        }
      });
      tabled_builder.push_record(record);
    }
    let mut table = tabled_builder.build();
    if let Some(terminal_width) = self.context.terminal_width() {
      table.with(Width::wrap(terminal_width).keep_words(true).priority(PriorityMax::new(true)));
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

  fn print_table_no_borders(&self) -> Result<(), DshCliError> {
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
    if let Some(terminal_width) = self.context.terminal_width() {
      table.with(Width::truncate(terminal_width).priority(PriorityMax::new(true)).suffix("..."));
    }
    table.with(Padding::new(0, 2, 0, 0));
    table.with(Style::empty());
    self.context.print(table.to_string());
    Ok(())
  }

  fn print_toml(&self) -> DshCliResult<()> {
    match self.simplified_values() {
      Some(simplified_values) => match toml::to_string_pretty(&simplified_values) {
        Ok(json) => {
          self.context.print(json);
          Ok(())
        }
        Err(error) => Err(error!("could not convert simplified values to toml ({})", error)),
      },
      None => match toml::to_string_pretty(&self.values) {
        Ok(json) => {
          self.context.print(json);
          Ok(())
        }
        Err(error) => Err(error!("could not convert values to toml ({})", error)),
      },
    }
  }

  fn print_toml_compact(&self) -> DshCliResult<()> {
    match self.simplified_values() {
      Some(simplified_values) => match toml::to_string(&simplified_values) {
        Ok(json) => {
          self.context.print(json);
          Ok(())
        }
        Err(error) => Err(error!("could not convert simplified values to toml compact ({})", error)),
      },
      None => match toml::to_string(&self.values) {
        Ok(json) => {
          self.context.print(json);
          Ok(())
        }
        Err(error) => Err(error!("could not convert values to toml compact ({})", error)),
      },
    }
  }

  fn print_yaml(&self) -> DshCliResult<()> {
    match self.simplified_values() {
      Some(simplified_values) => match serde_yaml::to_string(&simplified_values) {
        Ok(json) => {
          self.context.print(json);
          Ok(())
        }
        Err(error) => Err(error!("could not convert simplified values to yaml ({})", error)),
      },
      None => match serde_yaml::to_string(&self.values) {
        Ok(json) => {
          self.context.print(json);
          Ok(())
        }
        Err(error) => Err(error!("could not convert values to yaml ({})", error)),
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
      Some(self.values.iter().map(|(_, value)| *value).collect_vec())
    }
  }
}
