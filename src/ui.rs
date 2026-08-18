use windows::Win32::UI::WindowsAndMessaging::{
    IDYES, MB_ICONERROR, MB_ICONWARNING, MB_OK, MB_YESNO, MessageBoxW,
};
use windows::core::HSTRING;

pub fn error(msg: &str) {
    unsafe {
        let _ = MessageBoxW(
            None,
            &HSTRING::from(msg),
            &HSTRING::from("gkms_fl - Launch Error"),
            MB_OK | MB_ICONERROR,
        );
    }
}

pub fn warning(msg: &str) {
    unsafe {
        let _ = MessageBoxW(
            None,
            &HSTRING::from(msg),
            &HSTRING::from("gkms_fl - Warning"),
            MB_OK | MB_ICONWARNING,
        );
    }
}

pub fn confirm_open_dgp(msg: &str) -> bool {
    let text = format!(
        "{msg}\n\nOpen DMM GAME PLAYER now?\n\nSelect Yes to open it, or No to close this message."
    );
    unsafe {
        MessageBoxW(
            None,
            &HSTRING::from(text),
            &HSTRING::from("gkms_fl - Action Required"),
            MB_YESNO | MB_ICONWARNING,
        ) == IDYES
    }
}
