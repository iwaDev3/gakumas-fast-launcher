use windows::Win32::UI::WindowsAndMessaging::{IDYES, MB_ICONERROR, MB_OK, MB_YESNO, MessageBoxW};
use windows::core::HSTRING;

pub fn error(msg: &str) {
    unsafe {
        let _ = MessageBoxW(
            None,
            &HSTRING::from(msg),
            &HSTRING::from("gkms_fl"),
            MB_OK | MB_ICONERROR,
        );
    }
}

pub fn confirm_open_dgp(msg: &str) -> bool {
    let text = format!(
        "{msg}\n\nYes: start DMM GAME PLAYER now (uses dmm_proxy if set)\nNo: close (open DMM yourself)"
    );
    unsafe {
        MessageBoxW(
            None,
            &HSTRING::from(text),
            &HSTRING::from("gkms_fl"),
            MB_YESNO | MB_ICONERROR,
        ) == IDYES
    }
}
