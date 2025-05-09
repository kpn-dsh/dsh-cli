use crate::context::Context;
use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::OutputFormat;
use serde::Serialize;
use tabled::settings::peaker::PriorityMax;
use tabled::settings::{Padding, Width};
use tabled::{builder::Builder as TabledBuilder, settings::Style, Table};

pub struct UnitFormatter<'a, L: Label> {
  target_id: String,
  labels: &'a [L],
  target_label: Option<&'a str>,
  context: &'a Context,
}

impl<'a, L> UnitFormatter<'a, L>
where
  L: Label,
{
  pub fn new<T: Into<String>>(target_id: T, labels: &'a [L], target_label: Option<&'a str>, context: &'a Context) -> Self {
    Self { target_id: target_id.into(), labels, target_label, context }
  }

  pub fn print<V: SubjectFormatter<L> + Serialize>(&self, value: &V, default_output_format: Option<OutputFormat>) -> Result<(), String> {
    match self.context.output_format(default_output_format) {
      OutputFormat::Csv => self.print_csv(value),
      OutputFormat::Json => self.print_json(value),
      OutputFormat::JsonCompact => self.print_json_compact(value),
      OutputFormat::Plain => Err("plain unit print not yet implemented".to_string()),
      OutputFormat::Quiet => Ok(()),
      OutputFormat::Table => self.print_table(value),
      OutputFormat::TableNoBorder => self.print_table_no_borders(value),
      OutputFormat::Toml => self.print_toml(value),
      OutputFormat::TomlCompact => self.print_toml_compact(value),
      OutputFormat::Yaml => self.print_yaml(value),
    }
  }

  pub fn _print_non_serializable<V: SubjectFormatter<L>>(&self, value: &V, default_output_format: Option<OutputFormat>) -> Result<(), String> {
    match self.context.output_format(default_output_format) {
      OutputFormat::Csv => self.print_csv(value),
      OutputFormat::Json => Err("serialization to json is not supported for this type".to_string()),
      OutputFormat::JsonCompact => Err("serialization to compact json is not supported for this type".to_string()),
      OutputFormat::Plain => Err("plain unit print not yet implemented".to_string()),
      OutputFormat::Quiet => Ok(()),
      OutputFormat::Table => self.print_table(value),
      OutputFormat::TableNoBorder => self.print_table_no_borders(value),
      OutputFormat::Toml => Err("serialization to toml is not supported for this type".to_string()),
      OutputFormat::TomlCompact => Err("serialization to compact toml is not supported for this type".to_string()),
      OutputFormat::Yaml => Err("serialization to yaml is not supported for this type".to_string()),
    }
  }

  fn print_csv<V: SubjectFormatter<L>>(&self, value: &V) -> Result<(), String> {
    if self.context.show_headers() {
      self.context.print(
        self
          .labels
          .iter()
          .map(|label| self.context.csv_value(label.as_str_for_csv()))
          .collect::<Result<Vec<_>, _>>()?
          .join(self.context.csv_separator().as_str()),
      );
    }
    self.context.print(
      self
        .labels
        .iter()
        .map(|label| self.context.csv_value(value.value(label, self.target_id.as_str()).as_str()))
        .collect::<Result<Vec<_>, _>>()?
        .join(self.context.csv_separator().as_str()),
    );
    Ok(())
  }

  fn create_table<V: SubjectFormatter<L>>(&self, value: &V) -> Table {
    let mut tabled_builder = TabledBuilder::default();
    if let Some(target_label) = self.target_label {
      tabled_builder.push_record([target_label, self.target_id.as_str()]);
      for label in self.labels {
        if !label.is_target_label() && label.as_str_for_unit() != target_label {
          let value = value.value(label, self.target_id.as_str());
          let split_value = value.split("\n").collect::<Vec<_>>();
          let mut value_iterator = split_value.iter();
          if let Some(first_line) = value_iterator.next() {
            tabled_builder.push_record([label.as_str_for_unit(), first_line]);
          }
          for next_line in value_iterator {
            tabled_builder.push_record(["", next_line]);
          }
        }
      }
    } else {
      for label in self.labels {
        tabled_builder.push_record([label.as_str_for_unit(), value.value(label, self.target_id.as_str()).as_str()]);
      }
    }
    let mut table = tabled_builder.build();
    if let Some(terminal_width) = self.context.terminal_width() {
      table.with(Width::wrap(terminal_width).keep_words(true).priority(PriorityMax::new(true)));
    }
    table
  }

  fn print_json<V: SubjectFormatter<L> + Serialize>(&self, value: &V) -> Result<(), String> {
    match serde_json::to_string_pretty(value) {
      Ok(json) => {
        self.context.print(json);
        Ok(())
      }
      Err(error) => Err(format!("could not convert value to json ({})", error)),
    }
  }

  fn print_json_compact<V: SubjectFormatter<L> + Serialize>(&self, value: &V) -> Result<(), String> {
    match serde_json::to_string(value) {
      Ok(json) => {
        self.context.print(json);
        Ok(())
      }
      Err(error) => Err(format!("could not convert value to compact json ({})", error)),
    }
  }

  fn print_toml<V: SubjectFormatter<L> + Serialize>(&self, value: &V) -> Result<(), String> {
    match toml::to_string_pretty(value) {
      Ok(toml) => {
        self.context.print(toml);
        Ok(())
      }
      Err(error) => Err(format!("could not convert value to compact toml ({})", error)),
    }
  }

  fn print_toml_compact<V: SubjectFormatter<L> + Serialize>(&self, value: &V) -> Result<(), String> {
    match toml::to_string(value) {
      Ok(toml) => {
        self.context.print(toml);
        Ok(())
      }
      Err(error) => Err(format!("could not convert value to compact toml ({})", error)),
    }
  }

  fn print_yaml<V: SubjectFormatter<L> + Serialize>(&self, value: &V) -> Result<(), String> {
    match serde_yaml::to_string(value) {
      Ok(yaml) => {
        self.context.print(yaml);
        Ok(())
      }
      Err(error) => Err(format!("could not convert value to yaml ({})", error)),
    }
  }

  fn print_table<V: SubjectFormatter<L>>(&self, value: &V) -> Result<(), String> {
    let mut table = self.create_table(value);
    table.with(Padding::new(1, 1, 0, 0));
    table.with(Style::sharp());
    self.context.print(table.to_string());
    Ok(())
  }

  fn print_table_no_borders<V: SubjectFormatter<L>>(&self, value: &V) -> Result<(), String> {
    let mut table = self.create_table(value);
    table.with(Padding::new(0, 2, 0, 0));
    table.with(Style::empty());
    self.context.print(table.to_string());
    Ok(())
  }
}
