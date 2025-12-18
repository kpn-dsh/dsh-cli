use crate::cipher::{decrypt, encrypt};
use crate::directory::{delete_refresh_token, read_refresh_token, write_refresh_token};
use crate::{error, DshCliResult};
use dsh_api::platform::DshPlatform;
use openidconnect::RefreshToken;

/// Get stored refresh token
///
/// # Parameters
/// * `platform` - Platform for which the token is requested.
///
/// # Returns
/// `Some(RefreshToken)`
/// `None`
pub(crate) fn get_stored_refresh_token(platform: &DshPlatform) -> DshCliResult<Option<RefreshToken>> {
  match read_refresh_token(platform)? {
    Some(encrypted_refresh_token) => match decrypt(encrypted_refresh_token.as_str()) {
      Ok(refresh_token) => Ok(Some(RefreshToken::new(refresh_token))),
      Err(_) => {
        delete_refresh_token(platform)?;
        Err(error!("error reading refresh token for platform '{}', token deleted", platform))
      }
    },
    None => Ok(None),
  }
}

pub(crate) fn store_refresh_token(refresh_token: &RefreshToken, platform: &DshPlatform) -> DshCliResult<()> {
  write_refresh_token(platform, encrypt(refresh_token.secret())?)
}

pub(crate) fn delete_stored_refresh_token(platform: &DshPlatform) -> DshCliResult<()> {
  delete_refresh_token(platform).map(|_| ())
}
