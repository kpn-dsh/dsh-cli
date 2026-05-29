use crate::context::Context;
use crate::formatters::{Label, SubjectFormatter};
use crate::formatters::{OutputFormat, Value};
use crate::{err, error_map, DshCliResult};
use itertools::Itertools;
use serde::Serialize;
use tabled::settings::peaker::PriorityMax;
use tabled::settings::{Padding, Width};
use tabled::{builder::Builder as TabledBuilder, settings::Style, Table};

pub(crate) struct UnitFormatter<'a, L: Label> {
  target_id: String,
  labels: &'a [L],
  context: &'a Context,
}

impl<'a, L> UnitFormatter<'a, L>
where
  L: Label,
{
  pub(crate) fn new<T: Into<String>>(target_id: T, labels: &'a [L], context: &'a Context) -> Self {
    Self { target_id: target_id.into(), labels, context }
  }

  pub(crate) fn print<V: SubjectFormatter<L> + Serialize>(&self, value: &V, default_output_format: Option<OutputFormat>) -> DshCliResult<()> {
    match self.context.output_format(default_output_format) {
      OutputFormat::Csv => self.print_csv(value),
      OutputFormat::Json => self.print_json(value),
      OutputFormat::JsonCompact => self.print_json_compact(value),
      OutputFormat::Plain => err!("plain unit print not yet implemented"),
      OutputFormat::Quiet => Ok(()),
      OutputFormat::Table => self.print_table(value),
      OutputFormat::TableNoBorder => self.print_table_no_borders(value),
      OutputFormat::Toml => self.print_toml(value),
      OutputFormat::TomlCompact => self.print_toml_compact(value),
      OutputFormat::Yaml => self.print_yaml(value),
    }
  }

  pub(crate) fn print_non_serializable<V: SubjectFormatter<L>>(&self, value: &V, default_output_format: Option<OutputFormat>) -> DshCliResult<()> {
    match self.context.output_format(default_output_format) {
      OutputFormat::Csv => self.print_csv(value),
      OutputFormat::Json => err!("serialization to json is not supported for this type"),
      OutputFormat::JsonCompact => err!("serialization to compact json is not supported for this type"),
      OutputFormat::Plain => err!("plain unit print not yet implemented"),
      OutputFormat::Quiet => Ok(()),
      OutputFormat::Table => self.print_table(value),
      OutputFormat::TableNoBorder => self.print_table_no_borders(value),
      OutputFormat::Toml => err!("serialization to toml is not supported for this type"),
      OutputFormat::TomlCompact => err!("serialization to compact toml is not supported for this type"),
      OutputFormat::Yaml => err!("serialization to yaml is not supported for this type"),
    }
  }

  fn print_csv<V: SubjectFormatter<L>>(&self, value: &V) -> DshCliResult<()> {
    if !self.context.no_csv_headers() {
      self.context.println(
        self
          .labels
          .iter()
          .map(|label| self.context.csv_value(label.as_str_for_csv()))
          .collect::<Result<Vec<_>, _>>()?
          .join(self.context.csv_separator().as_str()),
      );
    }
    self.context.println(
      self
        .labels
        .iter()
        .map(|label| self.context.csv_value(value.value(label, self.target_id.as_str()).to_undecorated_string().as_str()))
        .collect::<Result<Vec<_>, _>>()?
        .join(self.context.csv_separator().as_str()),
    );
    Ok(())
  }

  fn print_json<V: SubjectFormatter<L> + Serialize>(&self, value: &V) -> DshCliResult<()> {
    let json = serde_json::to_string_pretty(value).map_err(error_map!("could not convert value to json ({})"))?;
    self.context.println(json);
    Ok(())
  }

  fn print_json_compact<V: SubjectFormatter<L> + Serialize>(&self, value: &V) -> DshCliResult<()> {
    let json = serde_json::to_string(value).map_err(error_map!("could not convert value to compact json ({})"))?;
    self.context.println(json);
    Ok(())
  }

  fn print_toml<V: SubjectFormatter<L> + Serialize>(&self, value: &V) -> DshCliResult<()> {
    let toml = toml::to_string_pretty(value).map_err(error_map!("could not convert value to compact toml ({})"))?;
    self.context.println(toml);
    Ok(())
  }

  fn print_toml_compact<V: SubjectFormatter<L> + Serialize>(&self, value: &V) -> DshCliResult<()> {
    let toml = toml::to_string(value).map_err(error_map!("could not convert value to compact toml ({})"))?;
    self.context.println(toml);
    Ok(())
  }

  fn print_yaml<V: SubjectFormatter<L> + Serialize>(&self, value: &V) -> DshCliResult<()> {
    let yaml = serde_yaml::to_string(value).map_err(error_map!("could not convert value to yaml ({})"))?;
    self.context.println(yaml);
    Ok(())
  }

  fn print_table<V: SubjectFormatter<L>>(&self, value: &V) -> DshCliResult<()> {
    let mut table = self.create_table(value);
    table.with(Padding::new(1, 1, 0, 0));
    table.with(Style::sharp());
    self.context.println(table.to_string());
    Ok(())
  }

  fn print_table_no_borders<V: SubjectFormatter<L>>(&self, value: &V) -> DshCliResult<()> {
    let mut table = self.create_table(value);
    table.with(Padding::new(0, 2, 0, 0));
    table.with(Style::empty());
    self.context.println(table.to_string());
    Ok(())
  }

  fn create_table<V: SubjectFormatter<L>>(&self, value: &V) -> Table {
    let target_label = self
      .labels
      .iter()
      .find(|label| label.is_target_label())
      .map(|target_label| target_label.as_str_for_unit())
      .unwrap_or("target id");
    let mut tabled_builder = TabledBuilder::default();
    tabled_builder.push_record([self.context.apply_target_label_style(target_label), self.context.apply_target_style(self.target_id.as_str())]);
    for label in self.labels {
      if !label.is_target_label() && label.as_str_for_unit() != target_label {
        let value = value.value(label, self.target_id.as_str());
        if !matches!(value, Value::Hide) {
          let decorated_string = value.to_decorated_string(self.context);
          let split_value = decorated_string.split("\n").collect_vec();
          let mut value_iterator = split_value.iter();
          if let Some(first_line) = value_iterator.next() {
            tabled_builder.push_record([self.context.apply_label_style(label.as_str_for_unit()), first_line.to_string()]);
          }
          for next_line in value_iterator {
            tabled_builder.push_record(["", next_line]);
          }
        }
      }
    }
    let mut table = tabled_builder.build();
    if let Some(terminal_width) = self.context.terminal_width() {
      table.with(Width::wrap(terminal_width).keep_words(true).priority(PriorityMax::new(true)));
    }
    table
  }
}
