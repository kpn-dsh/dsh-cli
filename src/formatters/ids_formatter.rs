use crate::context::Context;
use crate::formatters::OutputFormat;
use std::collections::HashMap;
use tabled::settings::peaker::PriorityMax;
use tabled::settings::{Padding, Width};
use tabled::{builder::Builder as TabledBuilder, settings::Style};

pub struct IdsFormatter<'a> {
  label: &'a str,
  ids: Vec<&'a str>,
  context: &'a Context,
}

impl<'a> IdsFormatter<'a> {
  /// # Creates a new `TargetIdFormatter`
  pub fn new<T: AsRef<str> + ?Sized>(label: &'a T, context: &'a Context) -> Self {
    Self { label: label.as_ref(), ids: vec![], context }
  }

  /// # Pushes target ids
  ///
  /// This method expects a slice containing target ids.
  pub fn push_target_ids<T: AsRef<str>>(&mut self, target_ids: &'a [T]) -> &Self {
    for target_id in target_ids {
      self.ids.push(target_id.as_ref());
    }
    self
  }

  pub fn _push_target_id<T: AsRef<str>>(&mut self, target_id: &'a T) -> &Self {
    self.ids.push(target_id.as_ref());
    self
  }

  pub fn _is_empty(&self) -> bool {
    self.ids.is_empty()
  }

  pub fn print(&self) -> Result<(), String> {
    match self.context.output_format {
      OutputFormat::Csv => {
        self.context.print(self.ids.join(","));
        Ok(())
      }

      OutputFormat::Json => match serde_json::to_string_pretty(&self.ids) {
        Ok(json) => {
          self.context.print(json);
          Ok(())
        }
        Err(error) => Err(format!("could not convert target ids to json ({})", error)),
      },

      OutputFormat::JsonCompact => match serde_json::to_string(&self.ids) {
        Ok(json) => {
          self.context.print(json);
          Ok(())
        }
        Err(error) => Err(format!("could not convert target ids to json compact ({})", error)),
      },

      OutputFormat::Plain => {
        self.context.print(self.ids.join("\n"));
        Ok(())
      }

      OutputFormat::Quiet => Ok(()),

      OutputFormat::Table => {
        let mut tabled_builder = TabledBuilder::default();
        tabled_builder.push_record([self.label]);
        for target_id in &self.ids {
          tabled_builder.push_record::<[&str; 1]>([target_id]);
        }
        let mut table = tabled_builder.build();
        if let Some(terminal_width) = self.context.terminal_width {
          table.with(Width::truncate(terminal_width).priority(PriorityMax::new(true)).suffix("..."));
        }
        table.with(Padding::new(1, 1, 0, 0));
        table.with(Style::sharp());
        self.context.print(table.to_string());
        Ok(())
      }

      OutputFormat::TableNoBorder => {
        let mut tabled_builder = TabledBuilder::default();
        tabled_builder.push_record([self.label]);
        for target_id in &self.ids {
          tabled_builder.push_record::<[&str; 1]>([target_id]);
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

      OutputFormat::Toml => match toml::to_string_pretty(&HashMap::from([(&self.label, &self.ids)])) {
        Ok(toml) => {
          self.context.print(toml);
          Ok(())
        }
        Err(error) => Err(format!("could not convert target ids to toml ({})", error)),
      },

      OutputFormat::TomlCompact => match toml::to_string(&HashMap::from([(&self.label, &self.ids)])) {
        Ok(toml) => {
          self.context.print(toml);
          Ok(())
        }
        Err(error) => Err(format!("could not convert target ids to toml compact ({})", error)),
      },

      OutputFormat::Yaml => match serde_yaml::to_string(&self.ids) {
        Ok(yaml) => {
          self.context.print(yaml);
          Ok(())
        }
        Err(error) => Err(format!("could not convert target ids to yaml ({})", error)),
      },
    }
  }
}
