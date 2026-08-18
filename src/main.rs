#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let result = std::panic::catch_unwind(gkmasfl::launch::run);
    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => handle_error(&e),
        Err(_) => {
            gkmasfl::diag::error("panic");
            show("Internal launcher error. See debug.log or gkms_fl.log next to gkms_fl.exe.");
        }
    }
}

fn handle_error(e: &gkmasfl::error::Error) {
    if e.offers_open_dgp() {
        if confirm_open_dgp(&e.user_message()) {
            let cfg = gkmasfl::config::load().unwrap_or_default();
            if let Err(spawn_err) =
                gkmasfl::spawn::spawn_dgp(cfg.dmm_proxy.as_deref(), cfg.dmm_path.as_deref())
            {
                show(&spawn_err.user_message());
            }
        }
        return;
    }
    show(&e.user_message());
}

#[cfg(windows)]
fn confirm_open_dgp(msg: &str) -> bool {
    gkmasfl::ui::confirm_open_dgp(msg)
}

#[cfg(not(windows))]
fn confirm_open_dgp(_msg: &str) -> bool {
    false
}

#[cfg(windows)]
fn show(msg: &str) {
    gkmasfl::ui::error(msg);
}

#[cfg(not(windows))]
fn show(msg: &str) {
    eprintln!("{msg}");
    std::process::exit(1);
}
