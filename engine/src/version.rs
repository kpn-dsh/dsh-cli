use std::fmt;
use std::fmt::{Display, Formatter};

use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
pub struct Version {
  major: u32,
  minor: u32,
  patch: u32,
}

impl Version {
  pub fn new(major: u32, minor: u32, patch: u32) -> Version {
    Version { major, minor, patch }
  }
}

impl Display for Version {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
  }
}

impl TryFrom<&str> for Version {
  type Error = String;

  fn try_from(representation: &str) -> Result<Self, Self::Error> {
    let parts = representation.split('.').collect::<Vec<&str>>();
    match parts.len() {
      1 => {
        let major = parts.first().unwrap().parse::<u32>().map_err(|_| "major is not a number".to_string())?;
        Ok(Version::new(major, 0, 0))
      }
      2 => {
        let major = parts.first().unwrap().parse::<u32>().map_err(|_| "major is not a number".to_string())?;
        let minor = parts.get(1).unwrap().parse::<u32>().map_err(|_| "minor is not a number".to_string())?;
        Ok(Version::new(major, minor, 0))
      }
      3 => {
        let major = parts.first().unwrap().parse::<u32>().map_err(|_| "major is not a number".to_string())?;
        let minor = parts.get(1).unwrap().parse::<u32>().map_err(|_| "minor is not a number".to_string())?;
        let patch = parts.get(2).unwrap().parse::<u32>().map_err(|_| "patch is not a number".to_string())?;
        Ok(Version::new(major, minor, patch))
      }
      _ => Err(format!("illegal version ({})", representation)),
    }
  }
}

struct DeserializeVersionFromStringVisitor;

impl<'de> Visitor<'de> for DeserializeVersionFromStringVisitor {
  type Value = Version;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a valid version representation")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: Error,
  {
    match Version::try_from(v) {
      Ok(version) => Ok(version),
      Err(_) => Err(E::invalid_value(Unexpected::Str(v), &self)),
    }
  }
}

pub fn deserialize_version_from_representation<'de, D>(deserializer: D) -> Result<Version, D::Error>
where
  D: Deserializer<'de>,
{
  deserializer.deserialize_any(DeserializeVersionFromStringVisitor)
}

pub fn serialize_version_to_representation<S>(version: &Version, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  serializer.serialize_str(version.to_string().as_str())
}

#[test]
fn test_try_from() {
  assert_eq!(Version::try_from("1").unwrap(), Version::new(1, 0, 0));
  assert_eq!(Version::try_from("1.2").unwrap(), Version::new(1, 2, 0));
  assert_eq!(Version::try_from("1.2.3").unwrap(), Version::new(1, 2, 3));
}
