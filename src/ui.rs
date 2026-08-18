use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    BS_DEFPUSHBUTTON, BS_PUSHBUTTON, DLGTEMPLATE, DS_CENTER, DS_MODALFRAME, DS_SETFONT,
    DialogBoxIndirectParamW, EndDialog, IDCANCEL, IDI_ERROR, IDI_WARNING, IDNO, IDOK, IDYES,
    LoadIconW, MB_ICONERROR, MB_ICONWARNING, MB_OK, MB_YESNO, MessageBoxW, STM_SETICON,
    SendDlgItemMessageW, WM_CLOSE, WM_COMMAND, WM_INITDIALOG, WS_CAPTION, WS_CHILD, WS_POPUP,
    WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
};
use windows::core::HSTRING;

const DIALOG_WIDTH: i16 = 310;
const DIALOG_MARGIN: i16 = 12;
const ICON_ID: i32 = 100;
const MESSAGE_ID: i32 = 101;
const ICON_SIZE: i16 = 20;
const MESSAGE_X: i16 = 44;
const MESSAGE_WIDTH: i16 = DIALOG_WIDTH - MESSAGE_X - DIALOG_MARGIN;
const BUTTON_WIDTH: i16 = 52;
const BUTTON_HEIGHT: i16 = 17;
const BUTTON_GAP: i16 = 8;
const FONT_POINT_SIZE: u16 = 12;
const WRAP_COLUMN: usize = 58;
const LINE_HEIGHT: i16 = 8;
const MIN_TEXT_HEIGHT: i16 = 32;
const MAX_TEXT_LINES: usize = 20;

const BUTTON_CLASS: u16 = 0x0080;
const STATIC_CLASS: u16 = 0x0082;
const SS_ICON: u32 = 0x0003;
const SS_NOPREFIX: u32 = 0x0080;

#[repr(isize)]
#[derive(Clone, Copy)]
enum DialogIcon {
    Error = 1,
    Warning = 2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DialogButtons {
    Ok,
    YesNo,
}

pub fn error(msg: &str) {
    let _ = show_dialog(
        "gkms_fl - Launch Error",
        msg,
        DialogIcon::Error,
        DialogButtons::Ok,
    );
}

pub fn warning(msg: &str) {
    let _ = show_dialog(
        "gkms_fl - Warning",
        msg,
        DialogIcon::Warning,
        DialogButtons::Ok,
    );
}

pub fn confirm_open_dgp(msg: &str) -> bool {
    let text = format!(
        "{msg}\n\nOpen DMM GAME PLAYER now?\n\nSelect Yes to open it, or No to close this message."
    );
    show_dialog(
        "gkms_fl - Action Required",
        &text,
        DialogIcon::Warning,
        DialogButtons::YesNo,
    ) == IDYES.0 as isize
}

fn show_dialog(title: &str, message: &str, icon: DialogIcon, buttons: DialogButtons) -> isize {
    let template = DialogTemplate::new(title, message, buttons);
    let result = unsafe {
        DialogBoxIndirectParamW(
            None,
            template.as_ptr(),
            None,
            Some(dialog_proc),
            LPARAM(icon as isize),
        )
    };
    if result != -1 {
        return result;
    }

    let icon_style = match icon {
        DialogIcon::Error => MB_ICONERROR,
        DialogIcon::Warning => MB_ICONWARNING,
    };
    let button_style = match buttons {
        DialogButtons::Ok => MB_OK,
        DialogButtons::YesNo => MB_YESNO,
    };
    unsafe {
        MessageBoxW(
            None,
            &HSTRING::from(message),
            &HSTRING::from(title),
            icon_style | button_style,
        )
        .0 as isize
    }
}

unsafe extern "system" fn dialog_proc(
    dialog: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> isize {
    match message {
        WM_INITDIALOG => {
            let icon_name = if lparam.0 == DialogIcon::Error as isize {
                IDI_ERROR
            } else {
                IDI_WARNING
            };
            if let Ok(icon) = unsafe { LoadIconW(None, icon_name) } {
                unsafe {
                    SendDlgItemMessageW(
                        dialog,
                        ICON_ID,
                        STM_SETICON,
                        WPARAM(icon.0 as usize),
                        LPARAM(0),
                    );
                }
            }
            1
        }
        WM_COMMAND => {
            let notification = ((wparam.0 >> 16) & 0xffff) as u16;
            let control_id = (wparam.0 & 0xffff) as i32;
            if notification == 0
                && (control_id == IDOK.0
                    || control_id == IDCANCEL.0
                    || control_id == IDYES.0
                    || control_id == IDNO.0)
            {
                let _ = unsafe { EndDialog(dialog, control_id as isize) };
                return 1;
            }
            0
        }
        WM_CLOSE => {
            let _ = unsafe { EndDialog(dialog, IDCANCEL.0 as isize) };
            1
        }
        _ => 0,
    }
}

struct DialogTemplate {
    storage: Vec<u32>,
    byte_len: usize,
}

impl DialogTemplate {
    fn new(title: &str, message: &str, buttons: DialogButtons) -> Self {
        let text_lines = estimated_text_lines(message).clamp(4, MAX_TEXT_LINES);
        let text_height = (text_lines as i16 * LINE_HEIGHT).clamp(MIN_TEXT_HEIGHT, i16::MAX);
        let button_y = DIALOG_MARGIN + text_height + DIALOG_MARGIN;
        let dialog_height = button_y + BUTTON_HEIGHT + DIALOG_MARGIN;
        let control_count = if buttons == DialogButtons::Ok { 3 } else { 4 };

        let mut template = Self {
            storage: Vec::with_capacity((message.encode_utf16().count() + 128).div_ceil(2)),
            byte_len: 0,
        };
        template.push_u32(
            WS_POPUP.0
                | WS_CAPTION.0
                | WS_SYSMENU.0
                | DS_MODALFRAME as u32
                | DS_SETFONT as u32
                | DS_CENTER as u32,
        );
        template.push_u32(0);
        template.push_u16(control_count);
        template.push_i16(0);
        template.push_i16(0);
        template.push_i16(DIALOG_WIDTH);
        template.push_i16(dialog_height);
        template.push_u16(0);
        template.push_u16(0);
        template.push_string(title);
        template.push_u16(FONT_POINT_SIZE);
        template.push_string("Segoe UI");

        template.push_control(
            WS_CHILD.0 | WS_VISIBLE.0 | SS_ICON,
            DIALOG_MARGIN,
            DIALOG_MARGIN,
            ICON_SIZE,
            ICON_SIZE,
            ICON_ID as u16,
            STATIC_CLASS,
            "",
        );
        template.push_control(
            WS_CHILD.0 | WS_VISIBLE.0 | SS_NOPREFIX,
            MESSAGE_X,
            DIALOG_MARGIN,
            MESSAGE_WIDTH,
            text_height,
            MESSAGE_ID as u16,
            STATIC_CLASS,
            message,
        );

        let right_button_x = DIALOG_WIDTH - DIALOG_MARGIN - BUTTON_WIDTH;
        match buttons {
            DialogButtons::Ok => template.push_control(
                WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0 | BS_DEFPUSHBUTTON as u32,
                right_button_x,
                button_y,
                BUTTON_WIDTH,
                BUTTON_HEIGHT,
                IDOK.0 as u16,
                BUTTON_CLASS,
                "OK",
            ),
            DialogButtons::YesNo => {
                let left_button_x = right_button_x - BUTTON_GAP - BUTTON_WIDTH;
                template.push_control(
                    WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0 | BS_DEFPUSHBUTTON as u32,
                    left_button_x,
                    button_y,
                    BUTTON_WIDTH,
                    BUTTON_HEIGHT,
                    IDYES.0 as u16,
                    BUTTON_CLASS,
                    "Yes",
                );
                template.push_control(
                    WS_CHILD.0 | WS_VISIBLE.0 | WS_TABSTOP.0 | BS_PUSHBUTTON as u32,
                    right_button_x,
                    button_y,
                    BUTTON_WIDTH,
                    BUTTON_HEIGHT,
                    IDNO.0 as u16,
                    BUTTON_CLASS,
                    "No",
                );
            }
        }
        template
    }

    fn as_ptr(&self) -> *const DLGTEMPLATE {
        self.storage.as_ptr().cast()
    }

    fn push_control(
        &mut self,
        style: u32,
        x: i16,
        y: i16,
        width: i16,
        height: i16,
        id: u16,
        class: u16,
        text: &str,
    ) {
        self.align_to_u32();
        self.push_u32(style);
        self.push_u32(0);
        self.push_i16(x);
        self.push_i16(y);
        self.push_i16(width);
        self.push_i16(height);
        self.push_u16(id);
        self.push_u16(0xffff);
        self.push_u16(class);
        self.push_string(text);
        self.push_u16(0);
    }

    fn align_to_u32(&mut self) {
        if self.byte_len % 4 != 0 {
            self.push_u16(0);
        }
    }

    fn push_string(&mut self, value: &str) {
        for word in value.encode_utf16() {
            self.push_u16(word);
        }
        self.push_u16(0);
    }

    fn push_i16(&mut self, value: i16) {
        self.push_u16(value as u16);
    }

    fn push_u16(&mut self, value: u16) {
        match self.byte_len % 4 {
            0 => self.storage.push(value as u32),
            2 => {
                let slot = self
                    .storage
                    .last_mut()
                    .expect("half-filled template word must exist");
                *slot |= (value as u32) << 16;
            }
            _ => unreachable!("dialog template writes are UTF-16 aligned"),
        }
        self.byte_len += 2;
    }

    fn push_u32(&mut self, value: u32) {
        assert_eq!(self.byte_len % 4, 0, "u32 template field is unaligned");
        self.storage.push(value);
        self.byte_len += 4;
    }
}

fn estimated_text_lines(message: &str) -> usize {
    message
        .split('\n')
        .map(|line| line.chars().count().max(1).div_ceil(WRAP_COLUMN))
        .sum()
}
