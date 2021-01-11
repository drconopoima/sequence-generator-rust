use std::time::{SystemTime, UNIX_EPOCH};

pub fn millis_from_custom_epoch(custom_epoch: SystemTime) -> u64 {
    SystemTime::now()
        .duration_since(custom_epoch)
        .expect("Error: Failed to create Timestamp from custom epoch.")
        .as_millis() as u64
}

fn main() {
    let millis_now = millis_from_custom_epoch(UNIX_EPOCH);
    println!("The current time in millis is {:#?}", millis_now)
}

#[cfg(test)]
mod tests {
    #[test]
    fn millis_from_epoch() {
        use super::*;
        use std::thread::sleep;
        use std::time::Duration;
        let time_now = SystemTime::now();
        let millis_start = time_now
            .duration_since(UNIX_EPOCH)
            .expect("Error: Failed to get current time as duration from epoch.")
            .as_millis() as u64;
        sleep(Duration::from_millis(50));
        // Test UNIX EPOCH
        let millis_after = millis_from_custom_epoch(UNIX_EPOCH);
        assert!((millis_after - millis_start) < 52);
        // Test a CUSTOM EPOCH since launching test
        let custom_epoch = UNIX_EPOCH
            .checked_add(Duration::from_millis(millis_start))
            .expect("Error: Failed to create custom epoch.");
        let millis_elapsed_time = time_now
            .elapsed()
            .expect("Error: Failed to get elapsed time.")
            .as_millis() as u64;
        let millis_custom_epoch_time = millis_from_custom_epoch(custom_epoch);
        assert!((millis_elapsed_time - millis_custom_epoch_time) < 2);
    }
}
