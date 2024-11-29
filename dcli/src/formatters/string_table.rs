use tabled::settings::peaker::PriorityMax;
use tabled::settings::Reverse;
use tabled::settings::Rotate;
use tabled::settings::{Padding, Width};
use tabled::{builder::Builder as TabledBuilder, settings::Style};
use termion::terminal_size;

use crate::DcliContext;

pub struct _StringTable<'a> {
  context: &'a DcliContext<'a>,
  tabled_builder: TabledBuilder,
}

impl<'a> _StringTable<'a> {
  pub fn _new(labels: &'a [&'a str], context: &'a DcliContext) -> Self {
    let mut tabled_builder = TabledBuilder::default();
    tabled_builder.push_record(labels.iter().map(|label| label.to_string()));
    Self { context, tabled_builder }
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

  pub fn _print_list(self) {
    let mut table = self.tabled_builder.build();
    if let Ok((columns, _rows)) = terminal_size() {
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

  pub fn _print_show(self) {
    let mut table = self.tabled_builder.build();
    if let Ok((columns, _rows)) = terminal_size() {
      table.with(Width::truncate(columns as usize).priority(PriorityMax).suffix("..."));
    }
    if self.context.hide_border {
      table.with(Padding::new(0, 2, 0, 0));
      table.with(Style::empty());
    } else {
      table.with(Padding::new(1, 1, 0, 0));
      table.with(Style::sharp());
    }
    table.with(Rotate::Left);
    table.with(Reverse::default());
    println!("{}", table);
  }
}
