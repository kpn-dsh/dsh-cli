use crate::context::Context;
use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::formatters::OutputFormat;
use serde::Serialize;
use std::marker::PhantomData;
use tabled::settings::peaker::PriorityMax;
use tabled::settings::{Padding, Width};
use tabled::{builder::Builder as TabledBuilder, settings::Style, Table};

pub struct UnitFormatter<'a, L: Label, V: SubjectFormatter<L>> {
  target_id: String,
  labels: &'a [L],
  target_label: Option<&'a str>,
  value: &'a V,
  context: &'a Context<'a>,
  phantom: PhantomData<&'a V>,
}

impl<'a, L, V> UnitFormatter<'a, L, V>
where
  L: Label,
  V: SubjectFormatter<L> + Serialize,
{
  pub fn new<T: Into<String>>(target_id: T, labels: &'a [L], target_label: Option<&'a str>, value: &'a V, context: &'a Context) -> Self {
    Self { target_id: target_id.into(), labels, target_label, value, context, phantom: PhantomData }
  }

  pub fn print(&self) -> Result<(), String> {
    match self.context.output_format {
      OutputFormat::Csv => {
        if self.context.show_labels {
          self.context.print(
            self
              .labels
              .iter()
              .map(|label| self.context.csv_value(label.as_str_for_csv()))
              .collect::<Result<Vec<_>, _>>()?
              .join(self.context.csv_separator.as_str()),
          );
        }
        self.context.print(
          self
            .labels
            .iter()
            .map(|label| self.context.csv_value(self.value.value(label, self.target_id.as_str()).as_str()))
            .collect::<Result<Vec<_>, _>>()?
            .join(self.context.csv_separator.as_str()),
        );
        Ok(())
      }

      OutputFormat::Json => match serde_json::to_string_pretty(self.value) {
        Ok(json) => {
          self.context.print(json);
          Ok(())
        }
        Err(error) => Err(format!("could not convert value to json ({})", error)),
      },

      OutputFormat::JsonCompact => match serde_json::to_string(self.value) {
        Ok(json) => {
          self.context.print(json);
          Ok(())
        }
        Err(error) => Err(format!("could not convert value to json compact ({})", error)),
      },

      OutputFormat::Plain => Err("csv unit print not yet implemented".to_string()),

      OutputFormat::Quiet => Ok(()),

      OutputFormat::Table => {
        let mut table = self.create_table();
        table.with(Padding::new(1, 1, 0, 0));
        table.with(Style::sharp());
        self.context.print(table.to_string());
        Ok(())
      }

      OutputFormat::TableNoBorder => {
        let mut table = self.create_table();
        table.with(Padding::new(0, 2, 0, 0));
        table.with(Style::empty());
        self.context.print(table.to_string());
        Ok(())
      }

      OutputFormat::Toml => match toml::to_string_pretty(self.value) {
        Ok(toml) => {
          self.context.print(toml);
          Ok(())
        }
        Err(error) => Err(format!("could not convert value to toml ({})", error)),
      },

      OutputFormat::TomlCompact => match toml::to_string(self.value) {
        Ok(toml) => {
          self.context.print(toml);
          Ok(())
        }
        Err(error) => Err(format!("could not convert value to toml compact ({})", error)),
      },

      OutputFormat::Yaml => match serde_yaml::to_string(self.value) {
        Ok(yaml) => {
          self.context.print(yaml);
          Ok(())
        }
        Err(error) => Err(format!("could not convert value to yaml ({})", error)),
      },
    }
  }

  fn create_table(&self) -> Table {
    let mut tabled_builder = TabledBuilder::default();
    if let Some(target_label) = self.target_label {
      tabled_builder.push_record([target_label, self.target_id.as_str()]);
      for label in self.labels {
        if !label.is_target_label() && label.as_str_for_unit() != target_label {
          tabled_builder.push_record([label.as_str_for_unit(), self.value.value(label, self.target_id.as_str()).as_str()]);
        }
      }
    } else {
      for label in self.labels {
        tabled_builder.push_record([label.as_str_for_unit(), self.value.value(label, self.target_id.as_str()).as_str()]);
      }
    }
    let mut table = tabled_builder.build();
    if let Some(terminal_width) = self.context.terminal_width {
      table.with(Width::truncate(terminal_width).priority(PriorityMax::new(true)).suffix("..."));
    }
    table
  }
}
