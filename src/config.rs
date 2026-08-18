use crate::error::Error;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

pub const FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    pub dmm_proxy: Option<String>,
    pub dmm_path: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    #[serde(default)]
    dmm_proxy: Option<String>,
    #[serde(default)]
    dmm_path: Option<String>,
}

pub fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe().ok()?.parent().map(PathBuf::from)
}

pub fn config_path() -> Option<PathBuf> {
    Some(exe_dir()?.join(FILE_NAME))
}

pub fn redact_proxy(url: &str) -> String {
    match url.find('@') {
        Some(at) => {
            let scheme = url.find("://").map(|i| i + 3).unwrap_or(0);
            format!("{}***{}", &url[..scheme], &url[at..])
        }
        None => url.to_string(),
    }
}

pub fn load() -> Result<Config, Error> {
    let Some(path) = config_path() else {
        return Ok(Config::default());
    };
    match fs::read_to_string(&path) {
        Ok(text) => parse_config(&text),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(source) => Err(Error::SettingsUnreadable { path, source }),
    }
}

pub fn parse_config(text: &str) -> Result<Config, Error> {
    let raw: RawConfig =
        toml::from_str(text).map_err(|source| Error::SettingsInvalid { source })?;
    let dmm_proxy = normalize_optional(raw.dmm_proxy);
    if let Some(url) = dmm_proxy.as_deref() {
        ureq::Proxy::new(url).map_err(|_| Error::ProxyInvalid)?;
    }
    Ok(Config {
        dmm_proxy,
        dmm_path: normalize_optional(raw.dmm_path),
    })
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|s| {
        let t = s.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_key_is_direct() {
        assert_eq!(parse_config("").unwrap(), Config::default());
        assert_eq!(parse_config("# comment\n").unwrap().dmm_proxy, None);
    }

    #[test]
    fn empty_string_is_direct() {
        assert_eq!(parse_config("dmm_proxy = \"\"\n").unwrap().dmm_proxy, None);
        assert_eq!(
            parse_config("dmm_proxy = \"   \"\n").unwrap().dmm_proxy,
            None
        );
    }

    #[test]
    fn http_proxy_ok() {
        let cfg = parse_config("dmm_proxy = \"http://127.0.0.1:7890\"\n").unwrap();
        assert_eq!(cfg.dmm_proxy.as_deref(), Some("http://127.0.0.1:7890"));
    }

    #[test]
    fn socks5h_proxy_ok() {
        let cfg = parse_config("dmm_proxy = \"socks5h://127.0.0.1:7891\"\n").unwrap();
        assert_eq!(cfg.dmm_proxy.as_deref(), Some("socks5h://127.0.0.1:7891"));
    }

    #[test]
    fn invalid_toml() {
        assert!(matches!(
            parse_config("dmm_proxy = "),
            Err(Error::SettingsInvalid { .. })
        ));
    }

    #[test]
    fn unknown_key() {
        assert!(matches!(
            parse_config("foo = 1\n"),
            Err(Error::SettingsInvalid { .. })
        ));
    }

    #[test]
    fn invalid_proxy_url() {
        assert!(matches!(
            parse_config("dmm_proxy = \"ftp://127.0.0.1:1\"\n"),
            Err(Error::ProxyInvalid)
        ));
    }

    #[test]
    fn redact_userinfo() {
        assert_eq!(
            redact_proxy("http://user:pass@127.0.0.1:7890"),
            "http://***@127.0.0.1:7890"
        );
        assert_eq!(
            redact_proxy("socks5h://127.0.0.1:20808"),
            "socks5h://127.0.0.1:20808"
        );
    }

    #[test]
    fn dmm_path_optional() {
        assert_eq!(parse_config("").unwrap().dmm_path, None);
        assert_eq!(parse_config("dmm_path = \"\"\n").unwrap().dmm_path, None);
        let cfg = parse_config(
            "dmm_path = \"C:\\\\Program Files\\\\DMMGamePlayer\\\\DMMGamePlayer.exe\"\n",
        )
        .unwrap();
        assert_eq!(
            cfg.dmm_path.as_deref(),
            Some(r"C:\Program Files\DMMGamePlayer\DMMGamePlayer.exe")
        );
    }
}
