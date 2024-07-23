use std::fmt::{Display, Formatter};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::config_dir_name;

pub mod dsh_service;
pub mod processor;
pub mod processor_config;
pub mod processor_descriptor;
pub mod processor_registry;

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

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ProcessorId(pub String);

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct JunctionId(pub String);

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ServiceId(pub String);

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ProfileId(pub String);

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ParameterId(pub String);

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

impl ProcessorId {
  pub fn new(id: &str) -> Self {
    if Self::is_valid(id) {
      ProcessorId(id.to_string())
    } else {
      panic!("invalid processor id '{}'", id)
    }
  }

  pub fn is_valid(id: &str) -> bool {
    lazy_static! {
      static ref PROCESSOR_ID_REGEX: Regex = Regex::new("^[a-z][a-z0-9_-]{1,50}$").unwrap();
    }
    PROCESSOR_ID_REGEX.is_match(id)
  }
}

impl TryFrom<&str> for ProcessorId {
  type Error = String;

  fn try_from(id: &str) -> Result<Self, Self::Error> {
    if Self::is_valid(id) {
      Ok(ProcessorId(id.to_string()))
    } else {
      Err(format!("invalid processor id '{}'", id))
    }
  }
}

impl TryFrom<String> for ProcessorId {
  type Error = String;

  fn try_from(id: String) -> Result<Self, Self::Error> {
    if Self::is_valid(id.as_str()) {
      Ok(ProcessorId(id))
    } else {
      Err(format!("invalid processor id '{}'", id))
    }
  }
}

impl Display for ProcessorId {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl JunctionId {
  pub fn new(id: &str) -> Self {
    if Self::is_valid(id) {
      JunctionId(id.to_string())
    } else {
      panic!("invalid junction id '{}'", id)
    }
  }

  pub fn is_valid(id: &str) -> bool {
    lazy_static! {
      static ref JUNCTION_ID_REGEX: Regex = Regex::new("^[a-z][a-z0-9_-]{1,50}$").unwrap();
    }
    JUNCTION_ID_REGEX.is_match(id)
  }
}

impl TryFrom<&str> for JunctionId {
  type Error = String;

  fn try_from(id: &str) -> Result<Self, Self::Error> {
    if Self::is_valid(id) {
      Ok(JunctionId(id.to_string()))
    } else {
      Err(format!("invalid junction id '{}'", id))
    }
  }
}

impl TryFrom<String> for JunctionId {
  type Error = String;

  fn try_from(id: String) -> Result<Self, Self::Error> {
    if Self::is_valid(&id) {
      Ok(JunctionId(id))
    } else {
      Err(format!("invalid junction id '{}'", id))
    }
  }
}

impl Display for JunctionId {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl ServiceId {
  pub fn new(id: &str) -> Self {
    if Self::is_valid(id) {
      ServiceId(id.to_string())
    } else {
      panic!("invalid service id '{}'", id)
    }
  }

  pub fn is_valid(id: &str) -> bool {
    lazy_static! {
      static ref SERVICE_ID_REGEX: Regex = Regex::new("^[a-z0-9]{1,20}$").unwrap();
    }
    SERVICE_ID_REGEX.is_match(id)
  }
}

impl TryFrom<&str> for ServiceId {
  type Error = String;

  fn try_from(id: &str) -> Result<Self, Self::Error> {
    if Self::is_valid(id) {
      Ok(ServiceId(id.to_string()))
    } else {
      Err(format!("invalid service id '{}'", id))
    }
  }
}

impl TryFrom<String> for ServiceId {
  type Error = String;

  fn try_from(id: String) -> Result<Self, Self::Error> {
    if Self::is_valid(id.as_str()) {
      Ok(ServiceId(id))
    } else {
      Err(format!("invalid service id '{}'", id))
    }
  }
}

impl Display for ServiceId {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl ProfileId {
  pub fn new(id: &str) -> Self {
    if Self::is_valid(id) {
      ProfileId(id.to_string())
    } else {
      panic!("invalid profile id '{}'", id)
    }
  }

  pub fn is_valid(id: &str) -> bool {
    lazy_static! {
      static ref PROFILE_ID_REGEX: Regex = Regex::new("^[a-z0-9]{1,20}$").unwrap();
    }
    PROFILE_ID_REGEX.is_match(id)
  }
}

impl TryFrom<&str> for ProfileId {
  type Error = String;

  fn try_from(id: &str) -> Result<Self, Self::Error> {
    if Self::is_valid(id) {
      Ok(ProfileId(id.to_string()))
    } else {
      Err(format!("invalid profile id '{}'", id))
    }
  }
}

impl TryFrom<String> for ProfileId {
  type Error = String;

  fn try_from(id: String) -> Result<Self, Self::Error> {
    if Self::is_valid(id.as_str()) {
      Ok(ProfileId(id))
    } else {
      Err(format!("invalid profile id '{}'", id))
    }
  }
}

impl Display for ProfileId {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl ParameterId {
  pub fn new(id: &str) -> Self {
    if Self::is_valid(id) {
      ParameterId(id.to_string())
    } else {
      panic!("invalid parameter id '{}'", id)
    }
  }

  pub fn is_valid(id: &str) -> bool {
    lazy_static! {
      static ref PARAMETER_ID_REGEX: Regex = Regex::new("^[a-z][a-z0-9_-]{1,30}$").unwrap();
    }
    PARAMETER_ID_REGEX.is_match(id)
  }
}

impl TryFrom<&str> for ParameterId {
  type Error = String;

  fn try_from(id: &str) -> Result<Self, Self::Error> {
    if Self::is_valid(id) {
      Ok(ParameterId(id.to_string()))
    } else {
      Err(format!("invalid parameter id '{}'", id))
    }
  }
}

impl TryFrom<String> for ParameterId {
  type Error = String;

  fn try_from(id: String) -> Result<Self, Self::Error> {
    if Self::is_valid(id.as_str()) {
      Ok(ParameterId(id))
    } else {
      Err(format!("invalid parameter id '{}'", id))
    }
  }
}

impl Display for ParameterId {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

pub(crate) fn processor_config_dir_name() -> String {
  format!("{}/processors", config_dir_name())
}
