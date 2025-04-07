use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Version {
  major: u32,
  minor: u32,
  patch: u32,
  postfix: Option<String>,
}

impl Version {
  pub fn new(major: u32, minor: u32, patch: u32, postfix: Option<String>) -> Version {
    Version { major, minor, patch, postfix }
  }
}

impl Display for Version {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self.postfix {
      Some(ref postfix) => write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, postfix),
      None => write!(f, "{}.{}.{}", self.major, self.minor, self.patch),
    }
  }
}

impl PartialOrd<Self> for Version {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Version {
  fn cmp(&self, other: &Self) -> Ordering {
    match (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch)) {
      Ordering::Less => Ordering::Less,
      Ordering::Equal => match (&self.postfix, &other.postfix) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(self_postfix), Some(other_postfix)) => self_postfix.cmp(other_postfix),
      },
      Ordering::Greater => Ordering::Greater,
    }
  }
}

lazy_static! {
  static ref VERSION_REGEX: Regex = Regex::new(r"^([0-9]+)(?:.([0-9]+))?(?:.([0-9]+))?(?:-([a-zA-Z][a-zA-Z0-9_-]*))?$").unwrap();
}

impl FromStr for Version {
  type Err = String;

  fn from_str(representation: &str) -> Result<Self, Self::Err> {
    match VERSION_REGEX.captures(representation) {
      Some(captures) => Ok(Version::new(
        captures.get(1).unwrap().as_str().parse::<u32>().unwrap(),
        captures.get(2).map(|m| m.as_str().parse::<u32>().unwrap()).unwrap_or(0),
        captures.get(3).map(|m| m.as_str().parse::<u32>().unwrap()).unwrap_or(0),
        captures.get(4).map(|m| m.as_str().to_string()),
      )),
      None => Err(format!("invalid version representation {}", representation)),
    }
  }
}

#[test]
fn test_correct_representations() {
  let correct_representations: Vec<(Vec<&str>, Version)> = vec![
    (vec!["0", "0.0", "0.0.0"], Version::new(0, 0, 0, None)),
    (vec!["0-beta", "0.0-beta", "0.0.0-beta"], Version::new(0, 0, 0, Some("beta".to_string()))),
    (vec!["1", "1.0", "1.0.0"], Version::new(1, 0, 0, None)),
    (vec!["1-beta", "1.0-beta", "1.0.0-beta"], Version::new(1, 0, 0, Some("beta".to_string()))),
    (vec!["1.2", "1.2.0"], Version::new(1, 2, 0, None)),
    (vec!["1.2-beta", "1.2.0-beta"], Version::new(1, 2, 0, Some("beta".to_string()))),
    (vec!["1.2.3"], Version::new(1, 2, 3, None)),
    (vec!["1.2.3-beta"], Version::new(1, 2, 3, Some("beta".to_string()))),
  ];
  for (representations, version) in correct_representations {
    for representation in representations {
      assert_eq!(Version::from_str(representation).unwrap(), version);
    }
  }
}

#[test]
fn test_incorrect_representations() {
  const INCORRECT_REPRESENTATIONS: [&str; 10] = ["", " ", ".", "0.", ".0", "0..0", "a", "0beta", "1.2beta", "1.2.3beta"];
  for representation in INCORRECT_REPRESENTATIONS {
    assert!(Version::from_str(representation).is_err());
  }
}

#[test]
fn test_partial_ordering() {
  let mut ordered_versions = vec![
    Version::new(1, 1, 1, Some("alpha".to_string())),
    Version::new(1, 1, 1, Some("beta".to_string())),
    Version::new(1, 1, 1, None),
    Version::new(1, 1, 2, Some("alpha".to_string())),
    Version::new(1, 1, 2, Some("beta".to_string())),
    Version::new(1, 1, 2, None),
    Version::new(1, 2, 1, Some("alpha".to_string())),
    Version::new(1, 2, 1, Some("beta".to_string())),
    Version::new(1, 2, 1, None),
    Version::new(1, 2, 2, Some("alpha".to_string())),
    Version::new(1, 2, 2, Some("beta".to_string())),
    Version::new(1, 2, 2, None),
    Version::new(2, 1, 1, Some("alpha".to_string())),
    Version::new(2, 1, 1, Some("beta".to_string())),
    Version::new(2, 1, 1, None),
  ]
  .into_iter();
  let mut less = ordered_versions.next().unwrap();
  for greater in ordered_versions {
    assert!(less < greater);
    assert_ne!(less, greater);
    less = greater;
  }
}
