use crate::error::Error;
use std::path::Path;

pub(crate) fn split_execute_args(execute_args: &str) -> Vec<&str> {
    execute_args.split_ascii_whitespace().collect()
}

fn append_win_arg(cmd: &mut String, arg: &str) {
    cmd.push(' ');
    if arg.chars().any(|c| c.is_ascii_whitespace()) {
        cmd.push('"');
        cmd.push_str(arg);
        cmd.push('"');
    } else {
        cmd.push_str(arg);
    }
}

pub(crate) fn dgp_chromium_args(spec: &str) -> Result<Vec<String>, Error> {
    let proxy = ureq::Proxy::new(spec).map_err(|_| Error::ProxyInvalid)?;
    let scheme = match proxy.protocol() {
        ureq::ProxyProtocol::Http | ureq::ProxyProtocol::Https => "http",
        ureq::ProxyProtocol::Socks4 | ureq::ProxyProtocol::Socks4A => "socks4",
        ureq::ProxyProtocol::Socks5 | ureq::ProxyProtocol::Socks5h => "socks5",
        _ => return Err(Error::ProxyInvalid),
    };
    let host = proxy.host().to_string();
    let port = proxy.port();
    let mut args = vec![format!("--proxy-server={scheme}://{host}:{port}")];
    if matches!(
        proxy.protocol(),
        ureq::ProxyProtocol::Socks5 | ureq::ProxyProtocol::Socks5h
    ) {
        args.push(format!(
            "--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE {host}"
        ));
    }
    Ok(args)
}

pub fn resolve_dgp_path(configured: Option<&str>) -> Option<std::path::PathBuf> {
    if let Some(raw) = configured {
        let p = std::path::PathBuf::from(raw);
        if p.is_file() {
            return Some(p);
        }
        let nested = p.join("DMMGamePlayer.exe");
        if nested.is_file() {
            return Some(nested);
        }
    }
    find_dgp_exe_default()
}

fn find_dgp_exe_default() -> Option<std::path::PathBuf> {
    const KEYS: &[&str] = &["PROGRAMFILES", "ProgramW6432", "PROGRAMFILES(X86)"];
    for key in KEYS {
        let Some(root) = std::env::var_os(key) else {
            continue;
        };
        let exe = std::path::PathBuf::from(root)
            .join("DMMGamePlayer")
            .join("DMMGamePlayer.exe");
        if exe.is_file() {
            return Some(exe);
        }
    }
    None
}

pub fn spawn_dgp(dmm_proxy: Option<&str>, dmm_exe: Option<&str>) -> Result<(), Error> {
    let exe = resolve_dgp_path(dmm_exe).ok_or(Error::DgpExeMissing)?;
    if exe.to_string_lossy().contains('"') {
        return Err(Error::SpawnFailed);
    }
    let extra = match dmm_proxy {
        Some(spec) => dgp_chromium_args(spec)?,
        None => Vec::new(),
    };
    let args: Vec<&str> = extra.iter().map(String::as_str).collect();
    let cwd = exe.parent().unwrap_or_else(|| Path::new("."));
    crate::diag::info(&format!(
        "spawn dgp exe={} proxy={} args={args:?}",
        exe.display(),
        dmm_proxy
            .map(crate::config::redact_proxy)
            .unwrap_or_else(|| "none".into())
    ));
    #[cfg(windows)]
    {
        spawn_windows(&exe, cwd, &args, false)
    }
    #[cfg(not(windows))]
    {
        let _ = (args, cwd);
        Err(Error::SpawnFailed)
    }
}

pub fn spawn_game(
    install_dir: &Path,
    exec_file_name: &str,
    execute_args: &str,
    is_administrator: bool,
) -> Result<(), Error> {
    let exe = install_dir.join(exec_file_name);
    if !exe.is_file() {
        return Err(Error::ExeMissing(exe));
    }
    let exe_display = exe.to_string_lossy();
    if exe_display.contains('"') {
        return Err(Error::SpawnFailed);
    }
    let args = split_execute_args(execute_args);
    crate::diag::info(&format!(
        "spawn exe={} admin={} arg_count={} cwd={}",
        exe.display(),
        is_administrator,
        args.len(),
        install_dir.display()
    ));
    #[cfg(windows)]
    {
        spawn_windows(&exe, install_dir, &args, is_administrator)
    }
    #[cfg(not(windows))]
    {
        let _ = (args, is_administrator);
        Err(Error::SpawnFailed)
    }
}

#[cfg(windows)]
fn spawn_windows(
    exe: &Path,
    install_dir: &Path,
    args: &[&str],
    is_administrator: bool,
) -> Result<(), Error> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        CreateProcessW, CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS, PROCESS_INFORMATION,
        STARTUPINFOW,
    };
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let exe_wide: Vec<u16> = exe
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let dir_wide: Vec<u16> = install_dir
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    if is_administrator {
        let args_only = args.join(" ");
        let args_wide: Vec<u16> = args_only.encode_utf16().chain(std::iter::once(0)).collect();
        let ret = unsafe {
            ShellExecuteW(
                None,
                w!("runas"),
                PCWSTR(exe_wide.as_ptr()),
                PCWSTR(args_wide.as_ptr()),
                PCWSTR(dir_wide.as_ptr()),
                SW_SHOWNORMAL,
            )
        };
        if ret.0 as isize <= 32 {
            return Err(Error::SpawnFailed);
        }
        return Ok(());
    }

    let exe_quoted = format!("\"{}\"", exe.display());
    let mut cmdline = exe_quoted;
    for arg in args {
        append_win_arg(&mut cmdline, arg);
    }
    let mut cmd_wide: Vec<u16> = cmdline.encode_utf16().chain(std::iter::once(0)).collect();

    let mut si = STARTUPINFOW::default();
    si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    let mut pi = PROCESS_INFORMATION::default();

    unsafe {
        CreateProcessW(
            PCWSTR(exe_wide.as_ptr()),
            Some(windows::core::PWSTR(cmd_wide.as_mut_ptr())),
            None,
            None,
            false,
            DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP,
            None,
            PCWSTR(dir_wide.as_ptr()),
            &si,
            &mut pi,
        )
        .map_err(|_| Error::SpawnFailed)?;
        let _ = CloseHandle(pi.hThread);
        let _ = CloseHandle(pi.hProcess);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_gakumas_args() {
        let args = split_execute_args("/viewer_id=a /open_id=b /pf_access_token=c");
        assert_eq!(
            args,
            [" /viewer_id=a", " /open_id=b", " /pf_access_token=c"]
                .iter()
                .map(|s| s.trim())
                .collect::<Vec<_>>()
        );
        assert_eq!(args.len(), 3);
        assert_eq!(args[0], "/viewer_id=a");
        assert_eq!(args[1], "/open_id=b");
        assert_eq!(args[2], "/pf_access_token=c");
    }

    #[test]
    fn split_drops_empty() {
        let args = split_execute_args("  /viewer_id=a  /open_id=b  ");
        assert_eq!(args, ["/viewer_id=a", "/open_id=b"]);
        assert!(args.iter().all(|t| !t.is_empty()));
    }

    #[test]
    fn chromium_http_proxy() {
        let args = dgp_chromium_args("http://127.0.0.1:7890").unwrap();
        assert_eq!(args, ["--proxy-server=http://127.0.0.1:7890"]);
    }

    #[test]
    fn chromium_socks5h_proxy() {
        let args = dgp_chromium_args("socks5h://127.0.0.1:20808").unwrap();
        assert_eq!(args[0], "--proxy-server=socks5://127.0.0.1:20808");
        assert!(args[1].contains("host-resolver-rules"));
        assert!(args[1].contains("EXCLUDE 127.0.0.1"));
    }

    #[test]
    fn quote_spaced_arg() {
        let mut cmd = String::from("\"app.exe\"");
        append_win_arg(&mut cmd, "--proxy-server=http://127.0.0.1:1");
        append_win_arg(
            &mut cmd,
            "--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE 127.0.0.1",
        );
        assert_eq!(
            cmd,
            "\"app.exe\" --proxy-server=http://127.0.0.1:1 \"--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE 127.0.0.1\""
        );
    }

    #[test]
    fn configured_dgp_file_or_dir() {
        let dir = std::env::temp_dir().join(format!("gkmasfl-dgp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("DMMGamePlayer.exe");
        std::fs::write(&exe, b"").unwrap();
        assert_eq!(
            resolve_dgp_path(Some(exe.to_str().unwrap())),
            Some(exe.clone())
        );
        assert_eq!(resolve_dgp_path(Some(dir.to_str().unwrap())), Some(exe));
        assert_eq!(
            resolve_dgp_path(Some("/no/such/DMMGamePlayer.exe")),
            find_dgp_exe_default()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
