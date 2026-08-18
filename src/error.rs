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
        match self {
            Error::DgpDirMissing => {
                "DMM GAME PLAYER data directory not found. Install DMM GAME PLAYER, sign in, then retry.\nExpected: %APPDATA%\\dmmgameplayer5"
                    .to_string()
            }
            Error::DgpDirUnavailable { .. } => {
                "DMM GAME PLAYER data directory could not be accessed. Check its permissions, then retry."
                    .to_string()
            }
            Error::DgpExeMissing => {
                "DMMGamePlayer.exe not found. Set dmm_path in config.toml to the exe (or its folder), or install DMM GAME PLAYER to %PROGRAMFILES%\\DMMGamePlayer\\DMMGamePlayer.exe"
                    .to_string()
            }
            Error::LocalStateMissing => {
                "Local State not found. Start official DMM GAME PLAYER once, sign in, then retry."
                    .to_string()
            }
            Error::LocalStateUnreadable { .. } => {
                "Local State could not be read. Close DMM GAME PLAYER, check the file permissions, then retry. Details are in the log next to gkms_fl.exe."
                    .to_string()
            }
            Error::EncryptedKeyInvalid => {
                "Could not read the DMM GAME PLAYER local encryption key. Sign in again in DMM GAME PLAYER after an upgrade. If the log shows APPB / v20, this launcher must be updated."
                    .to_string()
            }
            Error::DpapiFailed { .. } => {
                "Windows DPAPI decryption failed. Run gkms_fl.exe as the same Windows user that installed DMM GAME PLAYER (not another account, not a different elevated user)."
                    .to_string()
            }
            Error::AuthStoreMissing => {
                "Login credentials not found (authAccessTokenData.enc). Open DMM GAME PLAYER, sign in, then retry."
                    .to_string()
            }
            Error::AuthStoreUnreadable { .. } => {
                "Login credentials could not be read. Close DMM GAME PLAYER, check authAccessTokenData.enc permissions, then retry. Details are in the log next to gkms_fl.exe."
                    .to_string()
            }
            Error::AuthPrefixUnsupported { prefix } => {
                format!(
                    "DMM GAME PLAYER credential encryption format changed ({prefix}). This build cannot decrypt it. Update the launcher."
                )
            }
            Error::AuthDecryptFailed => {
                "Failed to decrypt login credentials. Open DMM GAME PLAYER, sign in again, then retry."
                    .to_string()
            }
            Error::AccessTokenMissing => {
                "No valid accessToken in the saved credentials. Open DMM GAME PLAYER, sign in again, then retry."
                    .to_string()
            }
            Error::ConfigMissing => {
                "dmmgame.cnf not found. Install Gakuen Idolmaster with DMM GAME PLAYER, then retry."
                    .to_string()
            }
            Error::ConfigInvalid { .. } => {
                "dmmgame.cnf is invalid. Repair the Gakuen Idolmaster installation in DMM GAME PLAYER, then retry. Details are in the log next to gkms_fl.exe."
                    .to_string()
            }
            Error::ConfigUnreadable { .. } => {
                "dmmgame.cnf could not be read. Close DMM GAME PLAYER, check the file permissions, then retry. Details are in the log next to gkms_fl.exe."
                    .to_string()
            }
            Error::GakumasNotInstalled => {
                "Gakuen Idolmaster (gakumas) is not installed in DMM GAME PLAYER (missing, not installed, or empty path). Install or repair it in DMM GAME PLAYER, then retry."
                    .to_string()
            }
            Error::GameTypeUnsupported(ty) => {
                format!(
                    "Unsupported gameType from dmmgame.cnf: {ty}. Expected GCL, ACL, AMAIN, or GMAIN. Reinstall the game with DMM GAME PLAYER or update this launcher."
                )
            }
            Error::TokenExpired => {
                "DMM login expired (token 203 / E210012). Open DMM GAME PLAYER, sign in again, then retry. This launcher does not refresh tokens."
                    .to_string()
            }
            Error::DeviceAuthRequired => {
                "DMM requires device authentication. Launch the game once with official DMM GAME PLAYER, then retry."
                    .to_string()
            }
            Error::AreaBlocked { result_code } => {
                format!(
                    "DMM rejected this network (result_code {result_code}; area / country block). Use a Japan-region connection or set dmm_proxy in config.toml next to gkms_fl.exe to a Japan HTTP/SOCKS proxy, then retry. Official DMM GAME PLAYER may already be using a different accelerated route. See debug.log or gkms_fl.log next to the exe."
                )
            }
            Error::VersionMismatch { local, latest } => {
                format!(
                    "Game version is outdated (local {local}, latest {latest}). This launcher does not download updates. Open DMM GAME PLAYER, update Gakuen Idolmaster, then retry."
                )
            }
            Error::DrmUnsupported => {
                "The launch response includes a DRM token. This launcher cannot start DRM-protected titles. Use official DMM GAME PLAYER."
                    .to_string()
            }
            Error::ExeMissing(path) => {
                format!(
                    "Game executable not found:\n{}\nReinstall Gakuen Idolmaster with DMM GAME PLAYER or check that dmmgame.cnf path is correct.",
                    path.display()
                )
            }
            Error::SpawnFailed { .. } => {
                "Failed to start gakumas.exe. Check that the install path has no quotes, the file is not blocked by antivirus, and if DMM asked for admin, approve the UAC prompt. See the log next to gkms_fl.exe."
                    .to_string()
            }
            Error::Http(_) => {
                "Could not reach the DMM launch API (network or proxy). Check the internet connection. If you use a proxy, set dmm_proxy in config.toml next to gkms_fl.exe (http://host:port or socks5h://host:port) and confirm that port is listening. Then retry. Details are in debug.log or gkms_fl.log."
                    .to_string()
            }
            Error::Api { result_code, error } => {
                let detail = if error.is_empty() {
                    String::new()
                } else {
                    format!(" DMM said: {error}.")
                };
                format!(
                    "DMM launch failed (result_code {result_code}).{detail} Retry after signing in with DMM GAME PLAYER. See debug.log or gkms_fl.log next to the exe."
                )
            }
            Error::LaunchParse => {
                "DMM launch API returned a response this launcher could not parse. Retry. If it persists, update the launcher or start the game from DMM GAME PLAYER. See the log next to gkms_fl.exe."
                    .to_string()
            }
            Error::DeviceRandomFailed => {
                "Could not generate a random device fingerprint. Retry. If it persists, see the log next to gkms_fl.exe."
                    .to_string()
            }
            Error::SettingsInvalid { .. } => {
                "config.toml next to gkms_fl.exe is invalid. Compare it with example_cfg.toml (dmm_proxy, dmm_path), then retry."
                    .to_string()
            }
            Error::SettingsUnreadable { .. } => {
                "config.toml next to gkms_fl.exe could not be read. Check its permissions, then retry. Details are in the log next to gkms_fl.exe."
                    .to_string()
            }
            Error::ProxyInvalid => {
                "Invalid dmm_proxy in config.toml. Use http://127.0.0.1:PORT or socks5h://127.0.0.1:PORT (SOCKS must be socks5h, not a random scheme). Leave it empty to connect directly."
                    .to_string()
            }
            Error::LogPathUnavailable | Error::LogWriteFailed { .. } => {
                "Could not write the log file next to gkms_fl.exe. Move the exe to a writable folder and retry."
                    .to_string()
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
