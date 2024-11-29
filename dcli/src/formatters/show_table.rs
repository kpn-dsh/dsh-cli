use std::marker::PhantomData;

use tabled::settings::peaker::PriorityMax;
use tabled::settings::{Padding, Width};
use tabled::{builder::Builder as TabledBuilder, settings::Style};
use termion::terminal_size;

use crate::formatters::formatter::{Label, SubjectFormatter};
use crate::DcliContext;

pub struct ShowTable<'a, L: Label, V: SubjectFormatter<L>> {
  _labels: &'a [L],
  context: &'a DcliContext<'a>,
  tabled_builder: TabledBuilder,
  phantom: PhantomData<&'a V>,
}

impl<'a, L, V> ShowTable<'a, L, V>
where
  L: Label,
  V: SubjectFormatter<L>,
{
  pub fn new(target_id: &str, subject: &V, labels: &'a [L], context: &'a DcliContext) -> Self {
    let mut tabled_builder = TabledBuilder::default();
    tabled_builder.push_record([
      subject
        .target_label()
        .map(|target_label| target_label.label_for_show().to_owned())
        .unwrap_or_default(),
      target_id.to_string(),
    ]);
    for label in labels {
      if !subject.target_label().is_some_and(|target_label| target_label == *label) {
        tabled_builder.push_record([label.label_for_show().to_string(), subject.value(label, target_id)]);
      }
    }
    Self { _labels: labels, context, tabled_builder, phantom: PhantomData }
  }

  pub fn _is_empty(&self) -> bool {
    self.tabled_builder.count_records() == 1
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
