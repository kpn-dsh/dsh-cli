use std::fmt::{Display, Formatter};
use std::ops::Deref;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::{config_dir_name, identifier};

pub mod dsh_service;
pub mod processor;
pub mod processor_config;
pub mod processor_descriptor;
pub mod processor_registry;

identifier!(
  "processor",
  JunctionId,
  "junction identifier",
  "^[a-z][a-z0-9_-]{1,50}$",
  "valid_junction_id",
  "invalid.junction.id"
);
identifier!(
  "processor",
  ParameterId,
  "parameter identifier",
  "^[a-z][a-z0-9_-]{1,30}$",
  "valid_parameter_id",
  "invalid.parameter.id"
);
identifier!(
  "processor",
  ProcessorId,
  "processor identifier",
  "^[a-z][a-z0-9]{0,19}$",
  "validprocessorid",
  "invalid_processor_id"
);
identifier!(
  "processor",
  ProfileId,
  "profile identifier",
  "^[a-z0-9]{1,20}$",
  "validprofileid",
  "invalid_profile_id"
);
identifier!(
  "processor",
  ServiceId,
  "service identifier",
  "^[a-z][a-z0-9]{0,19}$",
  "valid_service_id",
  "invalid.service.id"
);
identifier!(
  "processor",
  DshServiceName,
  "service name",
  "^[a-z][a-z0-9]{0,19}-[a-z][a-z0-9]{0,19}$",
  "validprofileid-validprocessorid",
  "validprofileid_validprocessorid"
);
identifier!("processor", TaskId, "task identifier", "^.*$", "valid_task_id", "invalid.task.id");

#[derive(Clone, Debug, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub enum ProcessorType {
  #[serde(rename = "dsh-service")]
  DshService,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ProcessorIdentifier {
  pub processor_type: ProcessorType,
  pub id: ProcessorId,
}

// impl From<ServiceName> for (ProcessorId, ServiceId) {

// fn from(id: ServiceName) -> Self {
//   lazy_static! {
//         static ref SERVICE_NAME_REGEX: regex::Regex = regex::Regex::new("^([a-z][a-z0-9]{0,19})-([a-z][a-z0-9]{0,19})$").unwrap();
//       }
//
//   for caps in SERVICE_NAME_REGEX.captures_iter(id.to_string().as_str()) {
//     let m = caps.get(0).unwrap();
//     new.push_str(&template[last_match..m.start()]);
//     let place_holder = PlaceHolder::try_from(caps.get(1).unwrap().as_str())?;
//     match template_mapping.get(&place_holder) {
//       Some(value) => {
//         new.push_str(value);
//       }
//       None => return Err(format!("template resolution failed because placeholder '{}' has no value", place_holder)),
//     }
//     last_match = m.end();
//   }
//
//
//
//
//   (ProcessorId::new(), ServiceId::new())
//   todo!()
// }
// }

impl Display for ProcessorType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match &self {
      ProcessorType::DshService => write!(f, "dsh-service"),
    }
  }
}

impl ProcessorType {
  fn description(&self) -> &str {
    match self {
      ProcessorType::DshService => "DSH service managed by the DSH platform",
    }
  }

  fn label(&self) -> &str {
    match self {
      ProcessorType::DshService => "DSH Service",
    }
  }
}

impl Display for ProcessorIdentifier {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", &self.id, &self.processor_type)
  }
}

pub(crate) fn processor_config_dir_name() -> String {
  format!("{}/processors", config_dir_name())
}
