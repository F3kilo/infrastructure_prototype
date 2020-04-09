use std::time::{Duration, Instant};
use std::thread::sleep;

pub fn wait_for(
    what: impl Fn() -> bool,
    timeout: Duration,
    check_period: Duration,
) -> bool {
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
