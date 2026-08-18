use std::io;
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("DGP directory missing")]
    DgpDirMissing,
    #[error("DGP directory unavailable at {}: {source}", path.display())]
    DgpDirUnavailable {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("DMMGamePlayer.exe missing")]
    DgpExeMissing,
    #[error("Local State missing")]
    LocalStateMissing,
    #[error("Local State unreadable at {}: {source}", path.display())]
    LocalStateUnreadable {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("encrypted_key invalid")]
    EncryptedKeyInvalid,
    #[error("DPAPI decrypt failed: {detail}")]
    DpapiFailed { detail: String },
    #[error("auth store missing")]
    AuthStoreMissing,
    #[error("auth store unreadable at {}: {source}", path.display())]
    AuthStoreUnreadable {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("unsupported auth prefix: {prefix}")]
    AuthPrefixUnsupported { prefix: String },
    #[error("auth decrypt failed")]
    AuthDecryptFailed,
    #[error("accessToken missing")]
    AccessTokenMissing,
    #[error("dmmgame.cnf missing")]
    ConfigMissing,
    #[error("dmmgame.cnf invalid: {source}")]
    ConfigInvalid {
        #[source]
        source: serde_json::Error,
    },
    #[error("dmmgame.cnf unreadable at {}: {source}", path.display())]
    ConfigUnreadable {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("gakumas not installed")]
    GakumasNotInstalled,
    #[error("unsupported gameType: {0}")]
    GameTypeUnsupported(String),
    #[error("DMM token expired")]
    TokenExpired,
    #[error("DMM device auth required")]
    DeviceAuthRequired,
    #[error("DMM area blocked result_code={result_code}")]
    AreaBlocked { result_code: i64 },
    #[error("version mismatch local={local} latest={latest}")]
    VersionMismatch { local: String, latest: String },
    #[error("DRM token unsupported")]
    DrmUnsupported,
    #[error("exe missing: {}", .0.display())]
    ExeMissing(PathBuf),
    #[error("spawn failed: {detail}")]
    SpawnFailed { detail: String },
    #[error("http error: {0}")]
    Http(String),
    #[error("DMM api result_code={result_code}: {error}")]
    Api { result_code: i64, error: String },
    #[error("launch response parse failed")]
    LaunchParse,
    #[error("device RNG failed")]
    DeviceRandomFailed,
    #[error("config.toml invalid: {source}")]
    SettingsInvalid {
        #[source]
        source: toml::de::Error,
    },
    #[error("config.toml unreadable at {}: {source}", path.display())]
    SettingsUnreadable {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("dmm_proxy invalid")]
    ProxyInvalid,
    #[error("log path unavailable")]
    LogPathUnavailable,
    #[error("log write failed at {}: {source}", path.display())]
    LogWriteFailed {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

impl Error {
    pub fn user_message(&self) -> String {
        let log_file = crate::diag::file_name();
        match self {
            Error::DgpDirMissing => concat!(
                "DMM GAME PLAYER data was not found.\n\n",
                "What to do:\n",
                "1. Install and open DMM GAME PLAYER.\n",
                "2. Sign in to your DMM account.\n",
                "3. Retry gkms_fl.\n\n",
                "Expected folder:\n",
                "%APPDATA%\\dmmgameplayer5"
            )
            .to_string(),
            Error::DgpDirUnavailable { path, source } => {
                format!(
                    "gkms_fl cannot access the DMM GAME PLAYER data folder.\n\nFolder:\n{}\n\nWhat to do:\nClose DMM GAME PLAYER, make sure this Windows account can read the folder, and retry.\n\nWindows reported:\n{source}",
                    path.display()
                )
            }
            Error::DgpExeMissing => concat!(
                "DMM GAME PLAYER could not be opened because DMMGamePlayer.exe was not found.\n\n",
                "What to do:\n",
                "Install DMM GAME PLAYER in its default location, or set dmm_path in config.toml beside gkms_fl.exe to the executable or its folder.\n\n",
                "Default location:\n",
                "%PROGRAMFILES%\\DMMGamePlayer\\DMMGamePlayer.exe"
            )
            .to_string(),
            Error::LocalStateMissing => concat!(
                "Required DMM GAME PLAYER encryption data was not found.\n\n",
                "What to do:\n",
                "Open DMM GAME PLAYER, sign in, close it, then retry gkms_fl.\n\n",
                "Missing file:\n",
                "Local State"
            )
            .to_string(),
            Error::LocalStateUnreadable { path, source } => {
                format!(
                    "gkms_fl cannot read DMM GAME PLAYER encryption data.\n\nFile:\n{}\n\nWhat to do:\nClose DMM GAME PLAYER, make sure this Windows account can read the file, and retry.\n\nWindows reported:\n{source}",
                    path.display()
                )
            }
            Error::EncryptedKeyInvalid => {
                "gkms_fl could not use DMM GAME PLAYER's local encryption key.\n\nWhat to do:\nOpen DMM GAME PLAYER and sign in again, then retry. If the problem continues, update gkms_fl because the credential format may have changed."
                    .to_string()
            }
            Error::DpapiFailed { detail } => {
                format!(
                    "Windows could not decrypt the saved DMM credentials for this account.\n\nWhat to do:\nRun gkms_fl from the same Windows account that signed in to DMM GAME PLAYER. Do not use \"Run as different user\".\n\nWindows reported:\n{detail}"
                )
            }
            Error::AuthStoreMissing => concat!(
                "DMM GAME PLAYER login data was not found.\n\n",
                "What to do:\n",
                "Open DMM GAME PLAYER, sign in, close it, then retry gkms_fl.\n\n",
                "Missing file:\n",
                "authAccessTokenData.enc"
            )
            .to_string(),
            Error::AuthStoreUnreadable { path, source } => {
                format!(
                    "gkms_fl cannot read the saved DMM login data.\n\nFile:\n{}\n\nWhat to do:\nClose DMM GAME PLAYER, make sure this Windows account can read the file, and retry.\n\nWindows reported:\n{source}",
                    path.display()
                )
            }
            Error::AuthPrefixUnsupported { .. } => {
                "gkms_fl does not support the current DMM GAME PLAYER credential format.\n\nWhat to do:\nUpdate gkms_fl. If you already have the latest release, launch the game through DMM GAME PLAYER for now."
                    .to_string()
            }
            Error::AuthDecryptFailed => {
                "Saved DMM GAME PLAYER login data could not be decrypted.\n\nWhat to do:\nOpen DMM GAME PLAYER, sign in again, close it, then retry gkms_fl."
                    .to_string()
            }
            Error::AccessTokenMissing => {
                "DMM GAME PLAYER login data does not contain a valid sign-in token.\n\nWhat to do:\nOpen DMM GAME PLAYER, sign in again, close it, then retry gkms_fl."
                    .to_string()
            }
            Error::ConfigMissing => concat!(
                "Gakuen Idolmaster installation data was not found.\n\n",
                "What to do:\n",
                "Open DMM GAME PLAYER and install or repair Gakuen Idolmaster, then retry gkms_fl.\n\n",
                "Missing file:\n",
                "dmmgame.cnf"
            )
            .to_string(),
            Error::ConfigInvalid { source } => {
                format!(
                    "DMM GAME PLAYER installation data is invalid or uses an unsupported format.\n\nWhat to do:\nOpen DMM GAME PLAYER and repair Gakuen Idolmaster. If the problem continues, update gkms_fl.\n\nParser reported:\n{source}\n\nMore details are in {log_file} beside gkms_fl.exe."
                )
            }
            Error::ConfigUnreadable { path, source } => {
                format!(
                    "gkms_fl cannot read the DMM GAME PLAYER installation data.\n\nFile:\n{}\n\nWhat to do:\nClose DMM GAME PLAYER, make sure this Windows account can read the file, and retry.\n\nWindows reported:\n{source}",
                    path.display()
                )
            }
            Error::GakumasNotInstalled => {
                "DMM GAME PLAYER does not report Gakuen Idolmaster as installed.\n\nWhat to do:\nInstall the game, or use DMM GAME PLAYER's repair option if it is already installed, then retry gkms_fl."
                    .to_string()
            }
            Error::GameTypeUnsupported(ty) => {
                format!(
                    "This Gakuen Idolmaster installation uses an unsupported game type: {ty}\n\nSupported types:\nGCL, ACL, AMAIN, GMAIN\n\nWhat to do:\nUpdate gkms_fl. If the problem continues, repair the game in DMM GAME PLAYER."
                )
            }
            Error::TokenExpired => {
                "Your DMM sign-in has expired.\n\nWhat to do:\nOpen DMM GAME PLAYER, sign in again, close it, then retry gkms_fl."
                    .to_string()
            }
            Error::DeviceAuthRequired => {
                "DMM must verify this device before the game can start.\n\nWhat to do:\nOpen DMM GAME PLAYER and launch Gakuen Idolmaster once. Close the game, then retry gkms_fl."
                    .to_string()
            }
            Error::AreaBlocked { result_code } => {
                format!(
                    "DMM blocked the launch request because this connection is outside a supported region.\n\nWhat to do:\nConnect through a Japan-region network and retry. To use a dedicated proxy, set dmm_proxy in config.toml beside gkms_fl.exe.\n\nProxy examples:\nhttp://127.0.0.1:7890\nsocks5h://127.0.0.1:7891\n\nDMM result code: {result_code}"
                )
            }
            Error::VersionMismatch { local, latest } => {
                format!(
                    "Gakuen Idolmaster must be updated before it can start.\n\nInstalled version: {local}\nRequired version: {latest}\n\nWhat to do:\nOpen DMM GAME PLAYER and update the game, then retry gkms_fl. gkms_fl cannot install game updates."
                )
            }
            Error::DrmUnsupported => {
                "DMM requires DRM handling that gkms_fl does not support.\n\nWhat to do:\nLaunch the game through DMM GAME PLAYER."
                    .to_string()
            }
            Error::ExeMissing(path) => {
                format!(
                    "The Gakuen Idolmaster executable was not found.\n\nExpected file:\n{}\n\nWhat to do:\nOpen DMM GAME PLAYER and repair or reinstall the game, then retry gkms_fl.",
                    path.display()
                )
            }
            Error::SpawnFailed { detail } => {
                format!(
                    "Windows could not start the requested program.\n\nWhat to do:\n1. Close any existing DMM GAME PLAYER or game process.\n2. Check whether antivirus or security software blocked the program.\n3. Retry, and approve the Windows UAC prompt if one appears.\n\nWindows reported:\n{detail}\n\nMore details are in {log_file} beside gkms_fl.exe."
                )
            }
            Error::Http(detail) => {
                format!(
                    "gkms_fl could not connect to the DMM launch service.\n\nWhat to do:\n1. Check your internet connection.\n2. If config.toml contains dmm_proxy, make sure the proxy is running and reachable.\n3. Retry.\n\nNetwork error:\n{detail}\n\nMore details are in {log_file} beside gkms_fl.exe."
                )
            }
            Error::Api {
                result_code,
                error,
            } => {
                let dmm_message = error.trim();
                let dmm_message = if dmm_message.is_empty() {
                    "No message provided."
                } else {
                    dmm_message
                };
                format!(
                    "DMM rejected the launch request.\n\nResult code: {result_code}\nDMM message: {dmm_message}\n\nWhat to do:\nRetry once. If it fails again, sign in to DMM GAME PLAYER and launch the game there.\n\nMore details are in {log_file} beside gkms_fl.exe."
                )
            }
            Error::LaunchParse => {
                format!(
                    "DMM returned launch data that this version of gkms_fl cannot understand.\n\nWhat to do:\nUpdate gkms_fl and retry. If you already have the latest release, launch the game through DMM GAME PLAYER.\n\nMore details are in {log_file} beside gkms_fl.exe."
                )
            }
            Error::DeviceRandomFailed => {
                format!(
                    "Windows could not generate a temporary device identifier.\n\nWhat to do:\nRetry gkms_fl. If the problem happens again, restart Windows and check {log_file} beside gkms_fl.exe."
                )
            }
            Error::SettingsInvalid { source } => {
                format!(
                    "config.toml contains invalid settings.\n\nWhat to do:\nCompare it with example_cfg.toml. If you do not need a custom proxy or DMM GAME PLAYER path, delete config.toml and retry.\n\nParser reported:\n{source}"
                )
            }
            Error::SettingsUnreadable { path, source } => {
                format!(
                    "gkms_fl cannot read config.toml.\n\nFile:\n{}\n\nWhat to do:\nMake sure this Windows account can read the file, or delete it if you do not need custom settings, then retry.\n\nWindows reported:\n{source}",
                    path.display()
                )
            }
            Error::ProxyInvalid => concat!(
                "The dmm_proxy value in config.toml is not a valid proxy URL.\n\n",
                "Use one of these forms:\n",
                "http://127.0.0.1:7890\n",
                "socks5h://127.0.0.1:7891\n\n",
                "To connect directly, remove dmm_proxy or leave it empty."
            )
            .to_string(),
            Error::LogPathUnavailable => {
                "gkms_fl could not determine where to create its diagnostic log.\n\nWhat to do:\nMove gkms_fl.exe to a normal local folder that you can write to, then retry."
                    .to_string()
            }
            Error::LogWriteFailed { path, source } => {
                format!(
                    "gkms_fl could not create its diagnostic log.\n\nFile:\n{}\n\nWhat to do:\nMove gkms_fl.exe to a writable folder, or fix the folder permissions, then retry.\n\nWindows reported:\n{source}",
                    path.display()
                )
            }
        }
    }

    pub fn offers_open_dgp(&self) -> bool {
        matches!(
            self,
            Error::DgpDirMissing
                | Error::DgpDirUnavailable { .. }
                | Error::LocalStateMissing
                | Error::LocalStateUnreadable { .. }
                | Error::EncryptedKeyInvalid
                | Error::AuthStoreMissing
                | Error::AuthStoreUnreadable { .. }
                | Error::AuthDecryptFailed
                | Error::AccessTokenMissing
                | Error::ConfigMissing
                | Error::ConfigInvalid { .. }
                | Error::ConfigUnreadable { .. }
                | Error::GakumasNotInstalled
                | Error::TokenExpired
                | Error::DeviceAuthRequired
                | Error::VersionMismatch { .. }
                | Error::DrmUnsupported
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_dgp_offers() {
        assert!(Error::TokenExpired.offers_open_dgp());
        assert!(Error::AuthStoreMissing.offers_open_dgp());
        assert!(
            Error::AuthStoreUnreadable {
                path: PathBuf::from("auth.enc"),
                source: io::Error::other("locked"),
            }
            .offers_open_dgp()
        );
        assert!(
            Error::VersionMismatch {
                local: "1".into(),
                latest: "2".into()
            }
            .offers_open_dgp()
        );
        assert!(!Error::Http("x".into()).offers_open_dgp());
        assert!(!Error::AreaBlocked { result_code: 803 }.offers_open_dgp());
        assert!(
            !Error::SpawnFailed {
                detail: "test".into()
            }
            .offers_open_dgp()
        );
    }
}
