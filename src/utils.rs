use std::thread::sleep;
use std::time::{Duration, Instant};
use tinyfiledialogs::MessageBoxIcon;

pub fn wait_for(
    what: impl Fn() -> bool,
    timeout: Option<Duration>,
    check_period: Duration,
) -> bool {
    let timeout = match timeout {
        Some(t) => t,
        None => Duration::from_secs(u64::max_value()),
    };
    let deadline = Instant::now() + timeout;
    loop {
        let start_check = Instant::now();
        if what() {
            return true;
        }
        let end_check = Instant::now();
        if end_check > deadline {
            return false;
        }
        let check_time = end_check - start_check;
        if check_time >= check_period {
            continue;
        }
        let sleep_duration = check_period - check_time;
        sleep(sleep_duration);
    }
}

pub fn show_error_message(title: &str, message: &str) {
    tinyfiledialogs::message_box_ok(title, message, MessageBoxIcon::Error);
}
