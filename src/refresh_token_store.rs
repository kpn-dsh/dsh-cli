use crate::cipher::{decrypt, encrypt};
use crate::{dsh_directory, REFRESH_TOKENS_SUBDIRECTORY};
use dsh_api::platform::DshPlatform;
use openidconnect::RefreshToken;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

/// Get stored refresh token
///
/// # Parameters
/// * `platform` - Platform for which the token is requested.
///
/// # Returns
/// `Ok(Some(RefreshToken))`
/// `Ok(None)`
pub fn get_stored_refresh_token(platform: &DshPlatform) -> Result<Option<RefreshToken>, String> {
  let refresh_token_file = refresh_token_pathbuf(platform)?;
  match fs::read_to_string(&refresh_token_file) {
    Ok(encrypted_refresh_token) => match decrypt(encrypted_refresh_token.as_str()) {
      Ok(refresh_token) => Ok(Some(RefreshToken::new(refresh_token))),
      Err(_) => {
        delete_stored_refresh_token(platform)?;
        Err(format!(
          "error reading refresh token from file {}, file deleted",
          refresh_token_file.to_string_lossy()
        ))
      }
    },
    Err(error) => match error.kind() {
      ErrorKind::NotFound => Ok(None),
      _ => Err(format!("error reading refresh token {}", refresh_token_file.to_string_lossy())),
    },
  }
}

pub fn store_refresh_token(refresh_token: &RefreshToken, platform: &DshPlatform) -> Result<(), String> {
  let refresh_token_file = refresh_token_pathbuf(platform)?;
  let encrypted_refresh_token = encrypt(refresh_token.secret())?;
  match fs::write(&refresh_token_file, encrypted_refresh_token) {
    Ok(_) => Ok(()),
    Err(error) => Err(format!("could not write refresh token to file {}: {}", refresh_token_file.to_string_lossy(), error)),
  }
}

pub fn delete_stored_refresh_token(platform: &DshPlatform) -> Result<(), String> {
  let refresh_token_file = refresh_token_pathbuf(platform)?;
  match fs::remove_file(&refresh_token_file) {
    Ok(_) => Ok(()),
    Err(error) => Err(format!("error deleting refresh token file {}: {}", refresh_token_file.to_string_lossy(), error)),
  }
}

/// Create refresh token `PathBuf` for platform
///
/// Create the [PathBuf] for a refresh token file for the provided `platform`.
/// The filename will be "[HOME]/.dsh_cli/refresh-tokens/[platform-name].token".
///
/// # Parameters
/// * `platform` - Platform for which the [PathBuf] is created.
///
/// # Returns
/// * `Ok(PathBuf)`
/// * `Err(None)`
fn refresh_token_pathbuf(platform: &DshPlatform) -> Result<PathBuf, String> {
  match dsh_directory()? {
    Some(dsh_directory) => Ok(dsh_directory.join(format!("{}/{}.token", REFRESH_TOKENS_SUBDIRECTORY, platform.name()))),
    None => Err("default settings, dsh cli directory is set to none".to_string()),
  }
}
