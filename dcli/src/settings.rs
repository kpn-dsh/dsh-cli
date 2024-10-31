use std::env;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::io::ErrorKind::NotFound;
use std::path::PathBuf;

use homedir::my_home;
use serde::{Deserialize, Serialize};

use dsh_api::platform::DshPlatform;
use dsh_api::platform::DshPlatform::NpLz;

use crate::arguments::Verbosity;
use crate::arguments::Verbosity::{Medium, Off};

const DCLI_DIRECTORY_ENV_VAR: &str = "DCLI_HOME";
const DEFAULT_USER_DCLI_DIRECTORY: &str = ".dcli";
const TARGETS_SUBDIRECTORY: &str = "targets";
const DEFAULT_DCLI_SETTINGS_FILENAME: &str = "settings.toml";
const TOML_FILENAME_EXTENSION: &str = "toml";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Settings {
  #[serde(rename = "show-execution-time", skip_serializing_if = "Option::is_none")]
  show_execution_time: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  verbosity: Option<Verbosity>,
  #[serde(rename = "no-border", skip_serializing_if = "Option::is_none")]
  no_border: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct Target {
  pub(crate) platform: DshPlatform,
  pub(crate) tenant: String,
  pub(crate) group_user_id: String,
  password: String
}

impl Display for Target {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}@{}", self.tenant, self.group_user_id, self.platform)
  }
}

pub(crate) fn read_settings(explicit_settings_filename: Option<&str>) -> Result<Option<Settings>, String> {
  match explicit_settings_filename {
    Some(explicit_name) => {
      log::debug!("read settings from explicit file '{}'", explicit_name);
      read_and_deserialize_from_toml_file::<Settings>(PathBuf::new().join(explicit_name))
    },
    None => {
      let default_settings_file = dcli_directory()?.join(DEFAULT_DCLI_SETTINGS_FILENAME);
      log::debug!("read settings from default file '{}'", default_settings_file.to_string_lossy());
      read_and_deserialize_from_toml_file::<Settings>(default_settings_file)
    }
  }
}

pub(crate) fn upsert_settings(explicit_settings_filename: Option<&str>, settings: &Settings) -> Result<(), String> {
  match explicit_settings_filename {
    Some(explicit_name) => {
      log::debug!("upsert explicit settings file '{}'", explicit_name);
      serialize_and_write_to_toml_file::<Settings>(PathBuf::new().join(explicit_name), settings)
    },
    None => {
      let default_settings_file = dcli_directory()?.join(DEFAULT_DCLI_SETTINGS_FILENAME);
      log::debug!("upsert default settings file '{}'", default_settings_file.to_string_lossy());
      serialize_and_write_to_toml_file::<Settings>(default_settings_file, settings)
    }
  }
}

pub(crate) fn all_targets() -> Result<Vec<Target>, String> {
  let mut targets = vec![];
  for target_file in fs::read_dir(targets_directory()?).map_err(|error| error.to_string())? {
    if let Ok(Some(target)) = read_and_deserialize_from_toml_file(target_file.map_err(|error| error.to_string())?.path()) {
      targets.push(target);
    }
  }
  targets.sort();
  Ok(targets)
}

pub(crate) fn delete_target(platform: &DshPlatform, tenant: &str) -> Result<(), String> {
  let target_file = target_file(platform, tenant)?;
  match delete_file(&target_file) {
    Ok(_) => log::debug!("target file '{}' successfully deleted", target_file.to_string_lossy()),
    Err(error) => log::debug!("error while deleting target file '{}' ({})", target_file.to_string_lossy(), error)
  }
  Ok(())
}

pub(crate) fn read_target(platform: &DshPlatform, tenant: &str) -> Result<Option<Target>, String> {
  let target_file = target_file(platform, tenant)?;
  log::debug!("read target file '{}'", target_file.to_string_lossy());
  read_and_deserialize_from_toml_file(target_file)
}

pub(crate) fn upsert_target(target: &Target) -> Result<(), String> {
  let target_file = target_file(&target.platform, &target.tenant)?;
  log::debug!("upsert target file '{}'", target_file.to_string_lossy());
  serialize_and_write_to_toml_file(target_file, target)
}

fn dcli_directory() -> Result<PathBuf, String> {
  let dcli_directory =  match env::var(DCLI_DIRECTORY_ENV_VAR) {
    Ok(dcli_directory) => PathBuf::new().join(dcli_directory),
    Err(_) => match my_home() {
      Ok(Some(user_home_directory)) => user_home_directory.join(DEFAULT_USER_DCLI_DIRECTORY),
      _ => {
        let message = format!("could not determine dcli directory name (check environment variable {})", DCLI_DIRECTORY_ENV_VAR);
        log::error!("{}", &message);
        return Err(message)
      }
    }
  };
  match fs::create_dir_all(&dcli_directory) {
    Ok(_) => {
      match fs::create_dir_all(dcli_directory.join(TARGETS_SUBDIRECTORY)) {
        Ok(_) => Ok(dcli_directory),
        Err(io_error) => {
          let message = format!("could not create dcli targets directory '{}' ({})", dcli_directory.join(TARGETS_SUBDIRECTORY).to_string_lossy(), io_error);
          log::error!("{}", &message);
          Err(message)
        }
      }
    },
    Err(io_error) => {
      let message = format!("could not create dcli directory '{}' ({})", dcli_directory.to_string_lossy(), io_error);
      log::error!("{}", &message);
      Err(message)
    }
  }
}

fn targets_directory() -> Result<PathBuf, String> {
  Ok(dcli_directory()?.join(TARGETS_SUBDIRECTORY))
}

fn target_file(platform: &DshPlatform, tenant: &str) -> Result<PathBuf, String> {
  Ok(targets_directory()?.join(format!("{}.{}.{}", platform, tenant, TOML_FILENAME_EXTENSION)))
}

fn delete_file(toml_file: &PathBuf) -> Result<(), String> {
  match fs::remove_file(toml_file) {
    Ok(_) => Ok(()),
    Err(io_error) => {
      let message = format!("could not delete file '{}' ({})", toml_file.to_string_lossy(), io_error);
      log::error!("{}", &message);
      Err(message)
    }
  }
}

fn read_and_deserialize_from_toml_file<T>(toml_file: PathBuf) -> Result<Option<T>, String>
where
  T: for<'de> Deserialize<'de>,
{
  match fs::read_to_string(&toml_file) {
    Ok(toml_string) => match toml::from_str::<T>(&toml_string) {
      Ok(deserialized_toml) => Ok(Some(deserialized_toml)),
      Err(de_error) => {
        let message = format!("could not deserialize file '{}' ({})", toml_file.to_string_lossy(), de_error.message());
        log::error!("{}", &message);
        Err(message)
      },
    },
    Err(io_error) => match io_error.kind() {
      NotFound => Ok(None),
      _ => {
        let message = format!("could not read file '{}'", toml_file.to_string_lossy());
        log::error!("{}", &message);
        Err(message)
      },
    },
  }
}

fn serialize_and_write_to_toml_file<T>(toml_file: PathBuf, data: &T) -> Result<(), String>
where
  T: Serialize,
{
  match toml::to_string(data) {
    Ok(toml_string) => {
      match fs::write(&toml_file, toml_string) {
        Ok(_) => Ok(()),
        Err(io_error) => {
          let message = format!("could not write file '{}' ({})", toml_file.to_string_lossy(), io_error);
          log::error!("{}", &message);
          Err(message)
        },
      }
    },
    Err(ser_error) => {
      let message = format!("could not serialize data ({})", ser_error);
      log::error!("{}", &message);
      Err(message)
    },
  }
}

fn _test_settings_filename() -> String {
  format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "../test_dcli_home/settings.toml")
}

#[test]
fn test_dcli_directory() {
  println!("{}", dcli_directory().unwrap().to_string_lossy());
}

#[test]
fn test_read_settings_default() {
  println!("settings default: {:?}", read_settings(None));
}

#[test]
fn test_read_settings_explicit_filename() {
  println!("settings explicit filename: {:?}", read_settings(Some(_test_settings_filename().as_str())));
}

#[test]
fn test_upsert_settings_default() {
  let settings = Settings {
    show_execution_time: Some(true),
    verbosity: Some(Off),
    no_border: None,
  };
  upsert_settings(None, &settings).unwrap();
}

#[test]
fn test_upsert_settings_explicit_filename() {
  let settings = Settings {
    show_execution_time: Some(true),
    verbosity: Some(Off),
    no_border: None,
  };
  upsert_settings(Some(_test_settings_filename().as_str()), &settings).unwrap();
}

#[test]
fn test_all_targets() {
  for target in all_targets().unwrap() {
    println!("{}", target);
  }
}

#[test]
fn test_delete_target() {
  delete_target(&NpLz, "greenbox-dev").unwrap()
}

#[test]
fn test_upsert_target() {
  let target = Target {
    platform: NpLz,
    tenant: "greenbox".to_string(),
    group_user_id: "2067:2067".to_string(),
    password: "abcdefghijklmnopqrstuvwxyz".to_string(),
  };
  upsert_target(&target).unwrap();
}

#[test]
fn test_read_target() {
  println!("{:?}",  read_target(&NpLz, "greenbox-dev").unwrap());
}

#[test]
fn test_serialize_and_write_to_toml_file() {
  let settings = Settings {
    show_execution_time: Some(true),
    verbosity: Some(Medium),
    no_border: None,
  };
  serialize_and_write_to_toml_file(PathBuf::new().join(_test_settings_filename()), &settings).unwrap();
}

#[test]
fn test_write_target() {
  let target = Settings {
    show_execution_time: Some(true),
    verbosity: Some(Medium),
    no_border: None,
  };
  serialize_and_write_to_toml_file(dcli_directory().unwrap().join("settings.toml"), &target).unwrap();
}
