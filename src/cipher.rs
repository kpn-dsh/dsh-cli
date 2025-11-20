use homedir::my_home;
use itertools::Itertools;
use std::fs;
use std::time::UNIX_EPOCH;

use sha2::Digest;
use sha2::Sha256;

use aes_gcm::aead::consts::U12;
use aes_gcm::aead::Aead;
use aes_gcm::aead::AeadCore;
use aes_gcm::aead::KeyInit;
use aes_gcm::aead::Nonce;
use aes_gcm::aead::OsRng;
use aes_gcm::aes::Aes256;
use aes_gcm::Key;
use aes_gcm::{Aes256Gcm, AesGcm};

use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;

pub fn encrypt(plain_text: &String) -> Result<String, String> {
  let cipher: AesGcm<Aes256, U12> = Aes256Gcm::new(&get_key());

  let nonce: Nonce<Aes256Gcm> = Aes256Gcm::generate_nonce(&mut OsRng);

  let encoded_nonce: String = STANDARD_NO_PAD.encode(nonce);

  let ciphertext: Vec<u8> = cipher
    .encrypt(&nonce, plain_text.as_bytes())
    .map_err(|error| format!("could not encrypt: {}", error))?;

  let encoded_cyphertext: String = STANDARD_NO_PAD.encode(ciphertext);

  Ok(format!("{}.{}", encoded_cyphertext, encoded_nonce))
}

pub fn decrypt(encoded_ciphertext_nonce: &str) -> Result<String, String> {
  let parts: Vec<&str> = encoded_ciphertext_nonce.split(".").collect();

  if parts.len() == 2 {
    let encoded_ciphertext: &[u8] = parts[0].as_bytes();

    let encoded_nonce = parts[1].as_bytes();

    let ciphertext: Vec<u8> = STANDARD_NO_PAD
      .decode(encoded_ciphertext)
      .map_err(|error| format!("could not decode ciphertext: {}", error))?;

    let decoded_nonce: Vec<u8> = STANDARD_NO_PAD
      .decode(encoded_nonce)
      .map_err(|error| format!("could not decode nonce: {}", error))?;

    #[allow(deprecated)] // TODO
    let nonce = Nonce::<Aes256Gcm>::clone_from_slice(&decoded_nonce);

    let cipher: AesGcm<Aes256, U12> = Aes256Gcm::new(&get_key());

    let plaintext: Vec<u8> = cipher
      .decrypt(&nonce, ciphertext.as_ref())
      .map_err(|error| format!("could not decrypt: {}", error))?;

    Ok(String::from_utf8(plaintext).map_err(|error| format!("could not create string: {}", error))?)
  } else {
    Err("illegal ciphertext nonce pair".to_string())
  }
}

pub fn get_key() -> Key<Aes256Gcm> {
  get_system_hash().unwrap().finalize()
}

/// Compute a hash based on unique system characteristics
///
/// Computes a shash value which should be unique for the system where the program is running.
/// The hash is computed by listing all the entries in the users home directory and generating
/// the hash from all entry names plus their creation timestamp.
///
/// Note that this algorithm does not guarantee that the computed has will be the same every
/// time the function is executed. Typically, if a new entry was added to the users home
/// directory or when an entry has been deleted, the computed hash will change.
pub fn get_system_hash() -> Result<Sha256, String> {
  match my_home() {
    Ok(Some(user_home_directory)) => match fs::read_dir(user_home_directory) {
      Ok(dir) => {
        let mut representations = dir
          .into_iter()
          .filter_map(|dir_entry| {
            dir_entry.ok().map(|entry| {
              // Representation is the concatenation file name and the creation timestamp
              format!(
                "{}:{}",
                entry.file_name().to_str().unwrap(),
                entry.metadata().unwrap().created().unwrap().duration_since(UNIX_EPOCH).unwrap().as_millis()
              )
            })
          })
          .collect_vec();
        representations.sort();
        let mut hasher = Sha256::new();
        for dd in &representations {
          hasher.update(dd.as_bytes());
        }
        Ok(hasher)
      }

      Err(_) => Err("could not determine user home directory".to_string()),
    },
    _ => Err("could not determine user home directory".to_string()),
  }
}
