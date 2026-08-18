use crate::error::Error;
use std::path::Path;

#[cfg(any(windows, test))]
fn append_win_arg(cmd: &mut String, arg: &str) {
    if !cmd.is_empty() {
        cmd.push(' ');
    }

    let quoted = arg.is_empty() || arg.bytes().any(|b| matches!(b, b' ' | b'\t'));
    if quoted {
        cmd.push('"');
    }

    let mut backslashes = 0usize;
    for ch in arg.chars() {
        if ch == '\\' {
            backslashes += 1;
            continue;
        }

        if ch == '"' {
            cmd.extend(std::iter::repeat_n('\\', backslashes * 2 + 1));
            cmd.push('"');
        } else {
            cmd.extend(std::iter::repeat_n('\\', backslashes));
            cmd.push(ch);
        }
        backslashes = 0;
    }

    let trailing = if quoted { backslashes * 2 } else { backslashes };
    cmd.extend(std::iter::repeat_n('\\', trailing));
    if quoted {
        cmd.push('"');
    }
}

#[cfg(any(windows, test))]
fn encode_win_args(args: &[&str]) -> String {
    let mut encoded = String::new();
    for arg in args {
        append_win_arg(&mut encoded, arg);
    }
    encoded
}

#[cfg(any(windows, test))]
fn create_process_command_line(exe: &Path, parameters: &str) -> String {
    let mut command_line = format!("\"{}\"", exe.display());
    if !parameters.is_empty() {
        command_line.push(' ');
        command_line.push_str(parameters);
    }
    command_line
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
        return Err(Error::SpawnFailed {
            detail: format!("DMMGamePlayer.exe path contains a quote: {}", exe.display()),
        });
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
        let parameters = encode_win_args(&args);
        spawn_windows(&exe, cwd, &parameters, false)
    }
    #[cfg(not(windows))]
    {
        let _ = (args, cwd);
        Err(Error::SpawnFailed {
            detail: "DMM GAME PLAYER process creation is only supported on Windows".into(),
        })
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
        return Err(Error::SpawnFailed {
            detail: format!("game executable path contains a quote: {}", exe.display()),
        });
    }
    crate::diag::info(&format!(
        "spawn exe={} admin={} parameters_len={} cwd={}",
        exe.display(),
        is_administrator,
        execute_args.len(),
        install_dir.display()
    ));
    #[cfg(windows)]
    {
        spawn_windows(&exe, install_dir, execute_args, is_administrator)
    }
    #[cfg(not(windows))]
    {
        let _ = (execute_args, is_administrator);
        Err(Error::SpawnFailed {
            detail: "game process creation is only supported on Windows".into(),
        })
    }
}

#[cfg(windows)]
fn spawn_windows(
    exe: &Path,
    install_dir: &Path,
    parameters: &str,
    is_administrator: bool,
) -> Result<(), Error> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        CREATE_NEW_PROCESS_GROUP, CreateProcessW, DETACHED_PROCESS, PROCESS_INFORMATION,
        STARTUPINFOW,
    };
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::core::{PCWSTR, w};

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
        let parameters_wide: Vec<u16> = parameters
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let ret = unsafe {
            ShellExecuteW(
                None,
                w!("runas"),
                PCWSTR(exe_wide.as_ptr()),
                PCWSTR(parameters_wide.as_ptr()),
                PCWSTR(dir_wide.as_ptr()),
                SW_SHOWNORMAL,
            )
        };
        if ret.0 as isize <= 32 {
            return Err(Error::SpawnFailed {
                detail: format!("ShellExecuteW returned {}", ret.0 as isize),
            });
        }
        return Ok(());
    }

    let command_line = create_process_command_line(exe, parameters);
    let mut command_wide: Vec<u16> = command_line
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let si = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };
    let mut pi = PROCESS_INFORMATION::default();

    unsafe {
        CreateProcessW(
            PCWSTR(exe_wide.as_ptr()),
            Some(windows::core::PWSTR(command_wide.as_mut_ptr())),
            None,
            None,
            false,
            DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP,
            None,
            PCWSTR(dir_wide.as_ptr()),
            &si,
            &mut pi,
        )
        .map_err(|source| Error::SpawnFailed {
            detail: format!("CreateProcessW for {} failed: {source}", exe.display()),
        })?;
        let _ = CloseHandle(pi.hThread);
        let _ = CloseHandle(pi.hProcess);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_parameters_are_preserved_verbatim() {
        let exe = Path::new(r"C:\Program Files\gakumas\gakumas.exe");
        for parameters in [
            "",
            "\"\"",
            "\"a b\"",
            "\"a  b\"",
            "\"a\tb\"",
            "a\\\"b",
            "a\\\\\\\"b",
            "\"C:\\path with space\\\\\"",
        ] {
            let expected = if parameters.is_empty() {
                format!("\"{}\"", exe.display())
            } else {
                format!("\"{}\" {parameters}", exe.display())
            };
            assert_eq!(
                create_process_command_line(exe, parameters),
                expected,
                "{parameters:?}"
            );
        }
    }

    #[test]
    fn encode_win_args_handles_quotes_and_backslashes() {
        fn encode(arg: &str) -> String {
            encode_win_args(&[arg])
        }

        assert_eq!(encode(""), "\"\"");
        assert_eq!(encode("a b"), "\"a b\"");
        assert_eq!(encode("a\"b"), "a\\\"b");
        assert_eq!(encode("a\\\"b"), "a\\\\\\\"b");
        assert_eq!(encode("C:\\path\\"), "C:\\path\\");
        assert_eq!(
            encode("C:\\path with space\\"),
            "\"C:\\path with space\\\\\""
        );
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
        let mut command_line = String::from("\"app.exe\"");
        append_win_arg(&mut command_line, "--proxy-server=http://127.0.0.1:1");
        append_win_arg(
            &mut command_line,
            "--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE 127.0.0.1",
        );
        assert_eq!(
            command_line,
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
