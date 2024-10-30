use std::env;
use std::fmt::Debug;
use std::fs;
use std::io::ErrorKind::NotFound;
use std::path::PathBuf;

use homedir::my_home;
use serde::{Deserialize, Serialize};

use dsh_api::platform::DshPlatform;
use dsh_api::platform::DshPlatform::NpLz;

use crate::arguments::Verbosity;
use crate::arguments::Verbosity::{High, Low, Medium, Off};

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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Target {
  platform: DshPlatform,
  tenant: String,
  group_user_id: String,
  password: String
}

pub(crate) fn read_settings(explicit_settings_filename: Option<&str>, verbosity: Verbosity) -> Result<Option<Settings>, String> {
  match explicit_settings_filename {
    Some(explicit_name) => read_and_deserialize_from_toml_file::<Settings>(PathBuf::new().join(explicit_name), verbosity),
    None => read_and_deserialize_from_toml_file::<Settings>(dcli_directory()?.join(DEFAULT_DCLI_SETTINGS_FILENAME), verbosity)
  }
}

pub(crate) fn upsert_settings(explicit_settings_filename: Option<&str>, settings: &Settings, verbosity: Verbosity) -> Result<(), String> {
  match explicit_settings_filename {
    Some(explicit_name) => serialize_and_write_to_toml_file::<Settings>(PathBuf::new().join(explicit_name), settings, verbosity),
    None => serialize_and_write_to_toml_file::<Settings>(dcli_directory()?.join(DEFAULT_DCLI_SETTINGS_FILENAME), settings, verbosity)
  }
}

pub(crate) fn read_target(platform: &DshPlatform, tenant: &str, verbosity: Verbosity) -> Result<Option<Target>, String> {
 read_and_deserialize_from_toml_file(target_file(platform, tenant)?, verbosity)
}

pub(crate) fn upsert_target(target: &Target, verbosity: Verbosity) -> Result<(), String> {
  serialize_and_write_to_toml_file(target_file(&target.platform, &target.tenant)?, target , verbosity)
}

fn dcli_directory() -> Result<PathBuf, String> {
  let dcli_directory =  match env::var(DCLI_DIRECTORY_ENV_VAR) {
    Ok(dcli_directory) => PathBuf::new().join(dcli_directory),
    Err(_) => match my_home() {
      Ok(Some(user_home_directory)) => user_home_directory.join(DEFAULT_USER_DCLI_DIRECTORY),
      _ => return Err(format!("could not determine dcli directory name (check environment variable {})", DCLI_DIRECTORY_ENV_VAR))
    }
  };
  match fs::create_dir_all(&dcli_directory) {
    Ok(_) => {
      match fs::create_dir_all(dcli_directory.join(TARGETS_SUBDIRECTORY)) {
        Ok(_) => Ok(dcli_directory),
        Err(io_error) => Err(format!("could not create dcli targets directory '{}' ({})", dcli_directory.join(TARGETS_SUBDIRECTORY).to_string_lossy(), io_error))
      }
    },
    Err(io_error) => Err(format!("could not create dcli directory '{}' ({})", dcli_directory.to_string_lossy(), io_error))
  }
}

fn target_file(platform: &DshPlatform, tenant: &str) -> Result<PathBuf, String> {
  Ok(dcli_directory()?.join(TARGETS_SUBDIRECTORY).join(format!("{}.{}.{}", platform, tenant, TOML_FILENAME_EXTENSION)))
}

fn read_and_deserialize_from_toml_file<T>(toml_file: PathBuf, verbosity: Verbosity) -> Result<Option<T>, String>
where
  T: for<'de> Deserialize<'de>,
{
  match fs::read_to_string(&toml_file) {
    Ok(toml_string) => match toml::from_str::<T>(&toml_string) {
      Ok(deserialized_toml) => {
        if verbosity >= Verbosity::High {
          println!("read file '{}'", toml_file.to_string_lossy());
        }
        Ok(Some(deserialized_toml))
      }
      Err(de_error) => Err(format!("could not deserialize file '{}' ({})", toml_file.to_string_lossy(), de_error.message())),
    },
    Err(io_error) => match io_error.kind() {
      NotFound => Ok(None),
      _ => Err(format!("could not read file '{}'", toml_file.to_string_lossy())),
    },
  }
}

fn serialize_and_write_to_toml_file<T>(toml_file: PathBuf, data: &T, _verbosity: Verbosity) -> Result<(), String>
where
  T: Serialize,
{
  match toml::to_string(data) {
    Ok(toml_string) => {
      match fs::write(&toml_file, toml_string) {
        Ok(_) => Ok(()),
        Err(io_error) => Err(format!("could not write file '{}' ({})", toml_file.to_string_lossy(), io_error)),
      }
    },
    Err(ser_error) => Err(format!("could not serialize data ({})", ser_error)),
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
  println!("settings default: {:?}", read_settings(None, High));
}

#[test]
fn test_read_settings_explicit_filename() {
  println!("settings explicit filename: {:?}", read_settings(Some(_test_settings_filename().as_str()), High));
}

#[test]
fn test_upsert_settings_default() {
  let settings = Settings {
    show_execution_time: Some(true),
    verbosity: Some(Off)
  };
  upsert_settings(None, &settings, High).unwrap();
}

#[test]
fn test_upsert_settings_explicit_filename() {
  let settings = Settings {
    show_execution_time: Some(true),
    verbosity: Some(Off)
  };
  upsert_settings(Some(_test_settings_filename().as_str()), &settings, High).unwrap();
}

#[test]
fn test_upsert_target() {
  let target = Target {
    platform: NpLz,
    tenant: "greenbox-dev".to_string(),
    group_user_id: "1903:1903".to_string(),
    password: "abcdefghijklmnopqrstuvwxyz".to_string(),
  };
  upsert_target(&target, High).unwrap();
}

#[test]
fn test_read_target() {
println!("{:?}",  read_target(&NpLz, "greenbox-dev", Low).unwrap());
}

#[test]
fn test_serialize_and_write_to_toml_file() {
  let settings = Settings {
    show_execution_time: Some(true),
    verbosity: Some(Medium)
  };
  let _ = serialize_and_write_to_toml_file(PathBuf::new().join(_test_settings_filename()), &settings, Medium);
}

#[test]
fn test_write_target() {
  let target = Settings {
    show_execution_time: Some(true),
    verbosity: Some(Medium)
  };
  let _ = serialize_and_write_to_toml_file(dcli_directory().unwrap().join("settings.toml"), &target, Medium);
}
