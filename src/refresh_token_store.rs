use crate::cipher::{decrypt, encrypt};
use crate::{dsh_directory, error, DshCliResult, REFRESH_TOKENS_SUBDIRECTORY};
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
/// `Some(RefreshToken)`
/// `None`
pub(crate) fn get_stored_refresh_token(platform: &DshPlatform) -> DshCliResult<Option<RefreshToken>> {
  let refresh_token_file = refresh_token_pathbuf(platform)?;
  match fs::read_to_string(&refresh_token_file) {
    Ok(encrypted_refresh_token) => match decrypt(encrypted_refresh_token.as_str()) {
      Ok(refresh_token) => Ok(Some(RefreshToken::new(refresh_token))),
      Err(_) => {
        delete_stored_refresh_token(platform)?;
        Err(error!(
          "error reading refresh token from file {}, file deleted",
          refresh_token_file.to_string_lossy()
        ))
      }
    },
    Err(error) => match error.kind() {
      ErrorKind::NotFound => Ok(None),
      _ => Err(error!("error reading refresh token {}", refresh_token_file.to_string_lossy())),
    },
  }
}

pub(crate) fn store_refresh_token(refresh_token: &RefreshToken, platform: &DshPlatform) -> DshCliResult<()> {
  let refresh_token_file = refresh_token_pathbuf(platform)?;
  let encrypted_refresh_token = encrypt(refresh_token.secret())?;
  fs::write(&refresh_token_file, encrypted_refresh_token)?;
  Ok(())
}

pub(crate) fn delete_stored_refresh_token(platform: &DshPlatform) -> DshCliResult<()> {
  fs::remove_file(&refresh_token_pathbuf(platform)?)?;
  Ok(())
}

/// Create refresh token `PathBuf` for platform
///
/// Create the [PathBuf] for a refresh token file for the provided `platform`.
/// The filename will be "$HOME/.dsh_cli/refresh-tokens/[platform-name].token".
///
/// # Parameters
/// * `platform` - Platform for which the [PathBuf] is created.
///
/// # Returns
/// * `PathBuf`
fn refresh_token_pathbuf(platform: &DshPlatform) -> DshCliResult<PathBuf> {
  match dsh_directory()? {
    Some(dsh_directory) => Ok(dsh_directory.join(format!("{}/{}.token", REFRESH_TOKENS_SUBDIRECTORY, platform.name()))),
    None => Err(error!("default settings, dsh cli directory is set to none")),
  }
}
