use crate::crypto;
use crate::error::Error;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use zeroize::{Zeroize, Zeroizing};

pub struct DgpPaths {
    pub root: PathBuf,
    pub local_state: PathBuf,
    pub auth_store: PathBuf,
    pub config: PathBuf,
}

pub struct GakumasInstall {
    pub game_type: String,
    pub version: String,
    pub path: PathBuf,
}

#[derive(Deserialize)]
struct AuthFile {
    #[serde(rename = "accessToken")]
    access_token: Option<String>,
}

#[derive(Deserialize)]
struct CnfFile {
    contents: Vec<CnfItem>,
}

#[derive(Deserialize)]
struct CnfItem {
    #[serde(rename = "productId")]
    product_id: String,
    #[serde(rename = "gameType")]
    game_type: String,
    detail: CnfDetail,
}

#[derive(Deserialize)]
struct CnfDetail {
    #[serde(default)]
    installed: bool,
    #[serde(default)]
    version: String,
    #[serde(default)]
    path: String,
}

pub fn dgp_paths() -> Result<DgpPaths, Error> {
    let appdata = std::env::var_os("APPDATA").ok_or(Error::DgpDirMissing)?;
    let root = PathBuf::from(appdata).join("dmmgameplayer5");
    let metadata = match fs::metadata(&root) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Err(Error::DgpDirMissing);
        }
        Err(source) => {
            return Err(Error::DgpDirUnavailable { path: root, source });
        }
    };
    if !metadata.is_dir() {
        return Err(Error::DgpDirMissing);
    }
    Ok(DgpPaths {
        local_state: root.join("Local State"),
        auth_store: root.join("authAccessTokenData.enc"),
        config: root.join("dmmgame.cnf"),
        root,
    })
}

pub fn read_os_crypt_key(local_state: &Path) -> Result<Zeroizing<Vec<u8>>, Error> {
    let text = match fs::read_to_string(local_state) {
        Ok(text) => text,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Err(Error::LocalStateMissing);
        }
        Err(source) => {
            return Err(Error::LocalStateUnreadable {
                path: local_state.to_path_buf(),
                source,
            });
        }
    };
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|_| Error::EncryptedKeyInvalid)?;
    let b64 = value
        .pointer("/os_crypt/encrypted_key")
        .and_then(|v| v.as_str())
        .ok_or(Error::EncryptedKeyInvalid)?;
    let mut decoded = STANDARD
        .decode(b64)
        .map_err(|_| Error::EncryptedKeyInvalid)?;
    if decoded.len() < 5 || &decoded[..5] != b"DPAPI" {
        decoded.zeroize();
        return Err(Error::EncryptedKeyInvalid);
    }
    let key = unprotect_os_crypt_key(&decoded[5..])?;
    decoded.zeroize();
    if key.len() != 16 && key.len() != 32 {
        return Err(Error::EncryptedKeyInvalid);
    }
    Ok(key)
}

#[cfg(windows)]
fn unprotect_os_crypt_key(payload: &[u8]) -> Result<Zeroizing<Vec<u8>>, Error> {
    crypto::dpapi_unprotect(payload)
}

#[cfg(not(windows))]
fn unprotect_os_crypt_key(_payload: &[u8]) -> Result<Zeroizing<Vec<u8>>, Error> {
    Err(Error::DpapiFailed {
        detail: "DPAPI is only available on Windows".into(),
    })
}

pub fn parse_access_token(bytes: &[u8]) -> Result<Zeroizing<String>, Error> {
    let file: AuthFile = serde_json::from_slice(bytes).map_err(|_| Error::AuthDecryptFailed)?;
    match file.access_token {
        Some(token) if !token.is_empty() => Ok(Zeroizing::new(token)),
        _ => Err(Error::AccessTokenMissing),
    }
}

pub fn read_access_token(auth_store: &Path, key: &[u8]) -> Result<Zeroizing<String>, Error> {
    let blob = match fs::read(auth_store) {
        Ok(blob) => blob,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Err(Error::AuthStoreMissing);
        }
        Err(source) => {
            return Err(Error::AuthStoreUnreadable {
                path: auth_store.to_path_buf(),
                source,
            });
        }
    };
    let prefix = blob.get(..3).unwrap_or(&[]);
    crate::diag::info(&format!(
        "auth blob_len={} prefix={}",
        blob.len(),
        String::from_utf8_lossy(prefix)
    ));
    let plaintext = crypto::decrypt_v10(key, &blob)?;
    crate::diag::debug(&format!("auth plaintext_len={}", plaintext.len()));
    parse_access_token(&plaintext)
}

pub fn parse_gakumas_cnf(json: &str) -> Result<GakumasInstall, Error> {
    let file: CnfFile =
        serde_json::from_str(json).map_err(|source| Error::ConfigInvalid { source })?;
    file.contents
        .into_iter()
        .find(|item| {
            item.product_id == "gakumas" && item.detail.installed && !item.detail.path.is_empty()
        })
        .map(|item| GakumasInstall {
            game_type: item.game_type,
            version: item.detail.version,
            path: PathBuf::from(item.detail.path),
        })
        .ok_or(Error::GakumasNotInstalled)
}

pub fn read_gakumas_install(config: &Path) -> Result<GakumasInstall, Error> {
    let text = match fs::read_to_string(config) {
        Ok(text) => text,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Err(Error::ConfigMissing);
        }
        Err(source) => {
            return Err(Error::ConfigUnreadable {
                path: config.to_path_buf(),
                source,
            });
        }
    };
    parse_gakumas_cnf(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"{
  "contents": [
    {
      "productId": "gakumas",
      "gameType": "GCL",
      "detail": {
        "installed": true,
        "version": "1.10.1",
        "path": "G:\\Game\\gakumas"
      }
    },
    {
      "productId": "umamusume",
      "gameType": "GCL",
      "detail": {
        "installed": true,
        "version": "1.0.0",
        "path": "G:\\Game\\umamusume"
      }
    }
  ]
}"#;

    #[test]
    fn parse_issue173_cnf() {
        let install = parse_gakumas_cnf(FIXTURE).expect("fixture");
        assert_eq!(install.game_type, "GCL");
        assert_eq!(install.version, "1.10.1");
        assert_eq!(install.path, PathBuf::from(r"G:\Game\gakumas"));
    }

    #[test]
    fn cnf_without_gakumas() {
        let json = r#"{"contents":[{"productId":"umamusume","gameType":"GCL","detail":{"installed":true,"version":"1","path":"X:\\g"}}]}"#;
        assert!(matches!(
            parse_gakumas_cnf(json),
            Err(Error::GakumasNotInstalled)
        ));
    }

    #[test]
    fn cnf_not_installed() {
        let json = r#"{"contents":[{"productId":"gakumas","gameType":"GCL","detail":{"installed":false,"version":"1.10.1","path":"G:\\Game\\gakumas"}}]}"#;
        assert!(matches!(
            parse_gakumas_cnf(json),
            Err(Error::GakumasNotInstalled)
        ));
    }

    #[test]
    fn cnf_empty_path() {
        let json = r#"{"contents":[{"productId":"gakumas","gameType":"GCL","detail":{"installed":true,"version":"1.10.1","path":""}}]}"#;
        assert!(matches!(
            parse_gakumas_cnf(json),
            Err(Error::GakumasNotInstalled)
        ));
    }

    #[test]
    fn cnf_invalid_json() {
        assert!(matches!(
            parse_gakumas_cnf("{"),
            Err(Error::ConfigInvalid { .. })
        ));
    }

    #[test]
    fn access_token_present() {
        let token = parse_access_token(br#"{"accessToken":"abc","refreshToken":"x"}"#).unwrap();
        assert_eq!(&*token, "abc");
    }

    #[test]
    fn access_token_missing() {
        assert!(matches!(
            parse_access_token(br#"{}"#),
            Err(Error::AccessTokenMissing)
        ));
    }

    #[test]
    fn access_token_empty() {
        assert!(matches!(
            parse_access_token(br#"{"accessToken":""}"#),
            Err(Error::AccessTokenMissing)
        ));
    }

    #[test]
    fn local_state_read_error_keeps_io_context() {
        let path = std::env::temp_dir().join(format!("gkmasfl-local-state-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();

        match read_os_crypt_key(&path) {
            Err(Error::LocalStateUnreadable {
                path: error_path,
                source,
            }) => {
                assert_eq!(error_path, path);
                assert_ne!(source.kind(), std::io::ErrorKind::NotFound);
            }
            other => panic!("expected LocalStateUnreadable, got {other:?}"),
        }

        fs::remove_dir_all(&path).unwrap();
    }
}
