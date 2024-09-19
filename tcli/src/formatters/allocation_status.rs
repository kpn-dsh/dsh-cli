use trifonius_dsh_api::types::AllocationStatus;

use crate::formatters::formatter::{Label, SubjectFormatter, TableBuilder};
use crate::formatters::notifications_to_string;
use crate::TcliContext;

#[derive(Eq, Hash, PartialEq)]
pub enum AllocationStatusLabel {
  DerivedFrom,
  Notifications,
  Provisioned,
  Target,
}

impl Label for AllocationStatusLabel {
  fn label_show(&self) -> &str {
    match self {
      Self::DerivedFrom => "derived from",
      Self::Notifications => "notifications",
      Self::Provisioned => "provisioned",
      Self::Target => "target id",
    }
  }
}

impl SubjectFormatter<AllocationStatusLabel> for AllocationStatus {
  fn value(&self, label: &AllocationStatusLabel, target_id: &str) -> String {
    match label {
      AllocationStatusLabel::DerivedFrom => self.clone().derived_from.unwrap_or_default(),
      AllocationStatusLabel::Notifications => {
        if self.notifications.is_empty() {
          "none".to_string()
        } else {
          notifications_to_string(&self.notifications)
        }
      }
      AllocationStatusLabel::Provisioned => self.provisioned.to_string(),
      AllocationStatusLabel::Target => target_id.to_string(),
    }
  }

  fn target_label(&self) -> Option<AllocationStatusLabel> {
    Some(AllocationStatusLabel::Target)
  }
}

pub static ALLOCATION_STATUS_LABELS: [AllocationStatusLabel; 4] =
  [AllocationStatusLabel::Target, AllocationStatusLabel::Provisioned, AllocationStatusLabel::DerivedFrom, AllocationStatusLabel::Notifications];

pub static DEFAULT_ALLOCATION_STATUS_LABELS: [AllocationStatusLabel; 4] =
  [AllocationStatusLabel::Target, AllocationStatusLabel::Provisioned, AllocationStatusLabel::DerivedFrom, AllocationStatusLabel::Notifications];

pub fn print_allocation_statuses(target_ids: Vec<String>, allocation_statuses: Vec<AllocationStatus>, context: &TcliContext) {
  let zipped = target_ids.into_iter().zip(allocation_statuses).collect::<Vec<(String, AllocationStatus)>>();
  let mut builder = TableBuilder::list(&DEFAULT_ALLOCATION_STATUS_LABELS, context);
  builder.values(&zipped);
  builder.print();
}

pub fn print_allocation_status(target_id: String, allocation_status: AllocationStatus, context: &TcliContext) {
  let mut builder = TableBuilder::show(&DEFAULT_ALLOCATION_STATUS_LABELS, context);
  builder.value(target_id, &allocation_status);
  builder.print();
}
