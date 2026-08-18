//! DMM wire contract — edit this file first when Game Player or the launch API changes.

use crate::device::DeviceFingerprint;
use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use zeroize::Zeroizing;

pub const PRODUCT_ID: &str = "gakumas";
pub const FALLBACK_EXE: &str = "gakumas.exe";
pub const API_ORIGIN: &str = "https://apidgp-gameplayer.games.dmm.com";
pub const LAUNCH_CL: &str = "https://apidgp-gameplayer.games.dmm.com/v5/r2/launch/cl";
pub const LAUNCH_PKG: &str = "https://apidgp-gameplayer.games.dmm.com/v5/launch/pkg";
pub const CLIENT_APP: &str = "DMMGamePlayer5";
pub const CLIENT_VERSION: &str = "5.3.25";
pub const ELECTRON_VERSION: &str = "34.3.0";
pub const USER_AGENT: &str = "DMMGamePlayer5-Win/5.3.25 Electron/34.3.0";
pub const GAME_OS: &str = "win";
pub const LAUNCH_TYPE: &str = "LIB";
pub const USER_OS: &str = "win";
pub const RESULT_OK: i64 = 100;
pub const RESULT_TOKEN: i64 = 203;

#[derive(Serialize)]
pub struct LaunchRequest<'a> {
    pub product_id: &'a str,
    pub game_type: &'a str,
    pub game_os: &'a str,
    pub launch_type: &'a str,
    pub mac_address: &'a str,
    pub hdd_serial: &'a str,
    pub motherboard: &'a str,
    pub user_os: &'a str,
}

#[derive(Deserialize, Debug)]
pub struct DmmEnvelope<T> {
    pub result_code: i64,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct LaunchData {
    pub exec_file_name: String,
    pub latest_version: String,
    pub execute_args: String,
    pub is_administrator: bool,
    #[serde(default)]
    pub drm_auth_token: Option<String>,
}

pub struct LaunchOk {
    pub exec_file_name: String,
    pub latest_version: String,
    pub execute_args: Zeroizing<String>,
    pub is_administrator: bool,
}

fn launch_url(game_type: &str) -> Result<&'static str, Error> {
    match game_type {
        "GCL" | "ACL" => Ok(LAUNCH_CL),
        "AMAIN" | "GMAIN" => Ok(LAUNCH_PKG),
        other => Err(Error::GameTypeUnsupported(other.to_string())),
    }
}

pub fn map_launch_envelope(env: DmmEnvelope<LaunchData>) -> Result<LaunchOk, Error> {
    let err = env.error.as_deref().unwrap_or("");
    if env.result_code == RESULT_TOKEN || err.contains("E210012") {
        return Err(Error::TokenExpired);
    }
    if err.to_ascii_lowercase().contains("authenticate device") {
        return Err(Error::DeviceAuthRequired);
    }
    if (801..=803).contains(&env.result_code) {
        return Err(Error::AreaBlocked {
            result_code: env.result_code,
        });
    }
    if env.result_code != RESULT_OK {
        return Err(Error::Api {
            result_code: env.result_code,
            error: env.error.unwrap_or_default(),
        });
    }
    let mut data = env.data.ok_or(Error::LaunchParse)?;
    if data.exec_file_name.is_empty() {
        data.exec_file_name = FALLBACK_EXE.to_string();
    }
    if data
        .drm_auth_token
        .as_deref()
        .is_some_and(|t| !t.is_empty())
    {
        return Err(Error::DrmUnsupported);
    }
    Ok(LaunchOk {
        exec_file_name: data.exec_file_name,
        latest_version: data.latest_version,
        execute_args: Zeroizing::new(data.execute_args),
        is_administrator: data.is_administrator,
    })
}

pub fn post_launch(
    token: &str,
    game_type: &str,
    device: &DeviceFingerprint,
    dmm_proxy: Option<&str>,
) -> Result<LaunchOk, Error> {
    let url = launch_url(game_type)?;
    crate::diag::info(&format!("dmm POST {url} game_type={game_type}"));
    crate::diag::debug(&format!(
        "dmm headers UA Client-App Client-Version actauth(len={}) Cookie=age_check_done=0",
        token.len()
    ));
    let proxy = match dmm_proxy {
        Some(spec) => Some(ureq::Proxy::new(spec).map_err(|_| Error::ProxyInvalid)?),
        None => None,
    };
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .proxy(proxy)
        .build()
        .into();
    let request = LaunchRequest {
        product_id: PRODUCT_ID,
        game_type,
        game_os: GAME_OS,
        launch_type: LAUNCH_TYPE,
        mac_address: &device.mac_address,
        hdd_serial: &device.hdd_serial,
        motherboard: &device.motherboard,
        user_os: USER_OS,
    };
    let mut response = agent
        .post(url)
        .header("User-Agent", USER_AGENT)
        .header("Client-App", CLIENT_APP)
        .header("Client-Version", CLIENT_VERSION)
        .header("Connection", "keep-alive")
        .header("Sec-Fetch-Site", "none")
        .header("Sec-Fetch-Mode", "no-cors")
        .header("Sec-Fetch-Dest", "empty")
        .header("Accept-Language", "ja")
        .header("Priority", "u=1, i")
        .header("actauth", token)
        .header("Cookie", "age_check_done=0")
        .send_json(&request)
        .map_err(|e| {
            crate::diag::error(&format!("dmm http {}", e));
            Error::Http(e.to_string())
        })?;
    crate::diag::debug(&format!("dmm http_status={}", response.status()));
    let envelope: DmmEnvelope<LaunchData> = response.body_mut().read_json().map_err(|e| {
        crate::diag::error(&format!("dmm json {e}"));
        Error::LaunchParse
    })?;
    crate::diag::info(&format!(
        "dmm result_code={} error={:?} has_data={}",
        envelope.result_code,
        envelope.error.as_deref().unwrap_or(""),
        envelope.data.is_some()
    ));
    if let Some(data) = envelope.data.as_ref() {
        crate::diag::info(&format!(
            "dmm exec={} latest={} admin={} drm={}",
            data.exec_file_name,
            data.latest_version,
            data.is_administrator,
            data.drm_auth_token
                .as_deref()
                .map(|s| !s.is_empty())
                .unwrap_or(false)
        ));
        crate::diag::debug(&format!("dmm execute_args_len={}", data.execute_args.len()));
    }
    map_launch_envelope(envelope)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_url_dispatch() {
        assert_eq!(launch_url("GCL").unwrap(), LAUNCH_CL);
        assert_eq!(launch_url("ACL").unwrap(), LAUNCH_CL);
        assert_eq!(launch_url("AMAIN").unwrap(), LAUNCH_PKG);
        assert_eq!(launch_url("GMAIN").unwrap(), LAUNCH_PKG);
        match launch_url("XXX") {
            Err(Error::GameTypeUnsupported(ty)) => assert_eq!(ty, "XXX"),
            other => panic!("expected GameTypeUnsupported, got {other:?}"),
        }
    }

    fn data() -> LaunchData {
        LaunchData {
            exec_file_name: "gakumas.exe".into(),
            latest_version: "1.10.1".into(),
            execute_args: "/viewer_id=a".into(),
            is_administrator: false,
            drm_auth_token: None,
        }
    }

    #[test]
    fn envelope_ok() {
        let ok = map_launch_envelope(DmmEnvelope {
            result_code: 100,
            data: Some(data()),
            error: None,
        })
        .unwrap();
        assert_eq!(ok.exec_file_name, "gakumas.exe");
        assert_eq!(&*ok.execute_args, "/viewer_id=a");
    }

    #[test]
    fn envelope_203_token_expired() {
        assert!(matches!(
            map_launch_envelope(DmmEnvelope {
                result_code: 203,
                data: None,
                error: None,
            }),
            Err(Error::TokenExpired)
        ));
    }

    #[test]
    fn envelope_e210012_token_expired() {
        assert!(matches!(
            map_launch_envelope(DmmEnvelope {
                result_code: 101,
                data: None,
                error: Some("E210012 token gone".into()),
            }),
            Err(Error::TokenExpired)
        ));
    }

    #[test]
    fn envelope_device_auth() {
        assert!(matches!(
            map_launch_envelope(DmmEnvelope {
                result_code: 100,
                data: Some(data()),
                error: Some("failed to authenticate device".into()),
            }),
            Err(Error::DeviceAuthRequired)
        ));
    }

    #[test]
    fn envelope_drm() {
        let mut d = data();
        d.drm_auth_token = Some("x".into());
        assert!(matches!(
            map_launch_envelope(DmmEnvelope {
                result_code: 100,
                data: Some(d),
                error: None,
            }),
            Err(Error::DrmUnsupported)
        ));
    }

    #[test]
    fn envelope_empty_exe_fallback() {
        let mut d = data();
        d.exec_file_name.clear();
        let ok = map_launch_envelope(DmmEnvelope {
            result_code: 100,
            data: Some(d),
            error: None,
        })
        .unwrap();
        assert_eq!(ok.exec_file_name, FALLBACK_EXE);
    }

    #[test]
    fn envelope_missing_data() {
        assert!(matches!(
            map_launch_envelope(DmmEnvelope {
                result_code: 100,
                data: None,
                error: None,
            }),
            Err(Error::LaunchParse)
        ));
    }

    #[test]
    fn envelope_other_api() {
        match map_launch_envelope(DmmEnvelope {
            result_code: 500,
            data: None,
            error: Some("boom".into()),
        }) {
            Err(Error::Api { result_code, error }) => {
                assert_eq!(result_code, 500);
                assert_eq!(error, "boom");
            }
            _ => panic!("expected Api"),
        }
    }

    #[test]
    fn envelope_area_blocked() {
        for code in [801_i64, 802, 803] {
            match map_launch_envelope(DmmEnvelope {
                result_code: code,
                data: None,
                error: None,
            }) {
                Err(Error::AreaBlocked { result_code }) => assert_eq!(result_code, code),
                _ => panic!("expected AreaBlocked({code})"),
            }
        }
    }
}
