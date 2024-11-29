use std::collections::HashMap;
use std::marker::PhantomData;

use tabled::settings::peaker::PriorityMax;
use tabled::settings::{Padding, Width};
use tabled::{builder::Builder as TabledBuilder, settings::Style};
use termion::terminal_size;

use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::DcliContext;

pub struct ListTable<'a, L: Label, V: SubjectFormatter<L>> {
  labels: &'a [L],
  context: &'a DcliContext<'a>,
  tabled_builder: TabledBuilder,
  phantom: PhantomData<&'a V>,
}

impl<'a, L, V> ListTable<'a, L, V>
where
  L: Label,
  V: SubjectFormatter<L>,
{
  pub fn new(labels: &'a [L], context: &'a DcliContext) -> Self {
    let mut tabled_builder = TabledBuilder::default();
    tabled_builder.push_record(labels.iter().map(|label| label.label_for_list()));
    Self { labels, context, tabled_builder, phantom: PhantomData }
  }

  pub fn is_empty(&self) -> bool {
    self.tabled_builder.count_records() == 1
  }

  pub fn _values(&mut self, values: &[(String, V)]) -> &Self {
    for (target_id, value) in values {
      self.tabled_builder.push_record(self.labels.iter().map(|label| value.value(label, target_id)));
    }
    self
  }

  pub fn value(&mut self, target_id: &str, value: &V) -> &Self {
    self.tabled_builder.push_record(self.labels.iter().map(|label| value.value(label, target_id)));
    self
  }

  pub fn _vecs(&mut self, vecs: &Vec<Vec<String>>) -> &Self {
    for vec in vecs {
      self.tabled_builder.push_record(vec);
    }
    self
  }

  pub fn _vec(&mut self, vec: &Vec<String>) -> &Self {
    self.tabled_builder.push_record(vec);
    self
  }

  pub fn _map(&mut self, map: &HashMap<String, V>) -> &Self {
    let mut target_ids = map.keys().collect::<Vec<&String>>();
    target_ids.sort();
    for target_id in target_ids {
      self
        .tabled_builder
        .push_record(self.labels.iter().map(|label| map.get(target_id).unwrap().value(label, target_id)));
    }
    self
  }

  pub fn _rows(&mut self, rows: &[V]) -> &Self {
    for row in rows {
      self
        .tabled_builder
        .push_record(self.labels.iter().map(|label| row.value(label, row.target_id().unwrap_or_default().as_str())));
    }
    self
  }

  pub fn row(&mut self, row: &V) -> &Self {
    self
      .tabled_builder
      .push_record(self.labels.iter().map(|label| row.value(label, row.target_id().unwrap_or_default().as_str())));
    self
  }

  pub fn print(self) {
    let mut table = self.tabled_builder.build();
    if let Ok((columns, _)) = terminal_size() {
      table.with(Width::truncate(columns as usize).priority(PriorityMax).suffix("..."));
    }
    if self.context.hide_border {
      table.with(Padding::new(0, 2, 0, 0));
      table.with(Style::empty());
    } else {
      table.with(Padding::new(1, 1, 0, 0));
      table.with(Style::sharp());
    }
    println!("{}", table);
  }
}
