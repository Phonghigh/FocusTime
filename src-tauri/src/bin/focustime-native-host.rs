//! Dedicated native-messaging-host binary. Chrome/Edge invoke whatever
//! executable is set as `"path"` in the registered native messaging host
//! manifest directly, with no way to pass it a custom CLI flag — so this
//! must be a separate binary from the main GUI app (`focustime.exe`), not
//! the same exe gated behind a `--native-host` flag. Point the manifest's
//! `"path"` at this binary's build output instead.

fn main() {
    focustime_lib::native_host::run();
}
