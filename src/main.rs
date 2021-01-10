use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn timestamp_from_epoch(custom_epoch: u64) -> u64 {
    let epoch = UNIX_EPOCH
        .checked_add(Duration::from_millis(custom_epoch))
        .expect("Error: Failed to create custom epoch.");
    SystemTime::now()
        .duration_since(epoch)
        .expect("Error: Failed to create Timestamp from custom epoch.")
        .as_millis() as u64
}

fn main() {
    let millis_now = timestamp_from_epoch(0);
    println!("The current time in millis is {:#?}", millis_now)
}

#[cfg(test)]
mod tests {
    #[test]
    fn timestamp_from_epoch() {
        use super::*;
        use std::thread::sleep;
        let time_now = SystemTime::now();
        let millis_start = time_now
            .duration_since(UNIX_EPOCH)
            .expect("Error: Failed to get current time as duration from epoch.")
            .as_millis() as u64;
        sleep(Duration::from_millis(50));
        // Test a CUSTOM EPOCH = UNIX EPOCH
        let millis_after = timestamp_from_epoch(0);
        assert!((millis_after - millis_start) < 52);
        // Test a CUSTOM EPOCH since launching test
        let millis_elapsed_time = time_now
            .elapsed()
            .expect("Error: Failed to get elapsed time.")
            .as_millis() as u64;
        let millis_custom_epoch_time = timestamp_from_epoch(millis_start);
        assert!((millis_elapsed_time - millis_custom_epoch_time) < 2);
    }
}
