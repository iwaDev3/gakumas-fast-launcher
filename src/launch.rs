use crate::config;
use crate::device;
use crate::dgp;
use crate::diag;
use crate::dmm;
use crate::error::Error;
use crate::spawn;

pub(crate) fn versions_match(local: &str, latest: &str) -> bool {
    local == latest
}

pub fn run() -> Result<(), Error> {
    diag::init()?;
    let result = run_inner();
    match &result {
        Ok(()) => diag::info("done"),
        Err(e) => diag::error(&format!("{e} | {}", e.user_message())),
    }
    result
}

fn run_inner() -> Result<(), Error> {
    let cfg = config::load()?;
    match config::config_path() {
        Some(p) if p.is_file() => diag::info(&format!("config {}", p.display())),
        Some(p) => diag::info(&format!("config missing (direct) {}", p.display())),
        None => diag::info("config path unavailable"),
    }
    match cfg.dmm_proxy.as_deref() {
        Some(p) => diag::info(&format!("dmm_proxy {}", config::redact_proxy(p))),
        None => diag::info("dmm_proxy none"),
    }
    match cfg.dmm_path.as_deref() {
        Some(p) => diag::info(&format!("dmm_path {p}")),
        None => diag::info("dmm_path none"),
    }

    let paths = dgp::dgp_paths()?;
    diag::info(&format!(
        "dgp root={} local_state={} auth={} cnf={}",
        paths.root.display(),
        paths.local_state.is_file(),
        paths.auth_store.is_file(),
        paths.config.is_file()
    ));

    let key = dgp::read_os_crypt_key(&paths.local_state)?;
    diag::info(&format!("os_crypt key_len={}", key.len()));
    let token = dgp::read_access_token(&paths.auth_store, &key)?;
    diag::info(&format!("accessToken len={}", token.len()));
    drop(key);

    let install = dgp::read_gakumas_install(&paths.config)?;
    diag::info(&format!(
        "gakumas type={} version={} path={}",
        install.game_type,
        install.version,
        install.path.display()
    ));

    let device = device::random_fingerprint()?;
    diag::info(&format!(
        "device mac={} hdd={} mb={}",
        device.mac_address, device.hdd_serial, device.motherboard
    ));

    let launch = dmm::post_launch(
        &token,
        &install.game_type,
        &device,
        cfg.dmm_proxy.as_deref(),
    )?;
    drop(token);
    diag::info(&format!(
        "launch exe={} latest={} admin={}",
        launch.exec_file_name, launch.latest_version, launch.is_administrator
    ));

    if !versions_match(&install.version, &launch.latest_version) {
        return Err(Error::VersionMismatch {
            local: install.version,
            latest: launch.latest_version,
        });
    }

    spawn::spawn_game(
        &install.path,
        &launch.exec_file_name,
        &launch.execute_args,
        launch.is_administrator,
    )?;
    drop(launch.execute_args);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_string_compare() {
        assert!(versions_match("1.0.0", "1.0.0"));
        assert!(!versions_match("1.0.0", "1.0.1"));
        assert!(!versions_match("1.10.1", "1.10.10"));
    }
}
