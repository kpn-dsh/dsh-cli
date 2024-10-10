use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
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

impl FromStr for Version {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::try_from(s)
  }
}

struct VersionStringVisitor;

impl<'de> Visitor<'de> for VersionStringVisitor {
  type Value = Version;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a version string 'x.y.z'")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: Error,
  {
    Version::try_from(value).map_err(|e| E::custom(e))
  }
}

impl<'de> Deserialize<'de> for Version {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_str(VersionStringVisitor)
  }
}

impl Serialize for Version {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    self.to_string().serialize(serializer)
  }
}

#[test]
fn test_try_from() {
  assert_eq!(Version::try_from("1").unwrap(), Version::new(1, 0, 0));
  assert_eq!(Version::try_from("1.2").unwrap(), Version::new(1, 2, 0));
  assert_eq!(Version::try_from("1.2.3").unwrap(), Version::new(1, 2, 3));
}

#[test]
fn test_deserialize() {
  #[derive(Deserialize)]
  struct StructureUnderTest {
    version: Version,
  }
  assert_eq!(
    serde_json::from_str::<StructureUnderTest>("{\"version\":\"1.2.3\"}").unwrap().version,
    Version::new(1, 2, 3)
  );
}

#[test]
fn test_serialize() {
  #[derive(Serialize)]
  struct StructureUnderTest {
    version: Version,
  }
  assert_eq!(
    serde_json::to_string(&StructureUnderTest { version: Version::new(1, 2, 3) }).unwrap(),
    "{\"version\":\"1.2.3\"}"
  );
}
