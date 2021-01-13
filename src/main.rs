use chrono::DateTime;
use dotenv;
use std::env;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(default_value = "1", long, short)]
    number: usize,
    #[structopt(default_value = "1970-01-01T00:00:00Z", long, short)]
    custom_epoch: String,
    #[structopt(default_value = "2", long)]
    micros_ten_power: u8,
    #[structopt(default_value = "9", long)]
    node_id_bits: u8,
    #[structopt(default_value = "11", long)]
    sequence_bits: u8,
    #[structopt(default_value = "0", long)]
    node_id: u16,
    #[structopt(default_value = "0", long)]
    sign_bits: u8,
    #[structopt(default_value = ".env", long)]
    dotenv_file: String,
    #[structopt(default_value = "1500", long)]
    cooldown_ns: u64,
    #[structopt(long, short)]
    debug: bool,
}

pub fn timestamp_from_custom_epoch(custom_epoch: SystemTime, micros_ten_power: u8) -> u64 {
    let timestamp;
    let mut micros_ten_power = micros_ten_power;
    if micros_ten_power >= 3 {
        timestamp = SystemTime::now()
            .duration_since(custom_epoch)
            .expect("Error: Failed to create Timestamp from custom epoch.")
            .as_millis();
        micros_ten_power -= 3;
    } else {
        timestamp = SystemTime::now()
            .duration_since(custom_epoch)
            .expect("Error: Failed to create Timestamp from custom epoch.")
            .as_micros();
    }
    let ten: u64 = 10;
    match micros_ten_power {
        0 => (timestamp as u64),
        _ => (timestamp as u64) / ten.pow(micros_ten_power.into()),
    }
}

pub struct SequenceProperties {
    pub sign_bits: u8,
    pub timestamp_bits: u8,
    pub node_id_bits: u8,
    pub sequence_bits: u8,
    pub custom_epoch: SystemTime,
    current_timestamp: Option<u64>,
    last_timestamp: Option<u64>,
    pub micros_ten_power: u8,
    pub node_id: u16,
    pub sequence: u16,
    pub max_sequence: u16,
    pub backoff_cooldown_start_ns: u64,
}

impl SequenceProperties {
    pub fn new(
        custom_epoch: SystemTime,
        node_id_bits: u8,
        node_id: u16,
        sequence_bits: u8,
        micros_ten_power: u8,
        sign_bits: u8,
        backoff_cooldown_start_ns: u64,
    ) -> Self {
        let timestamp_bits = (64 as u8)
            .checked_sub(sequence_bits)
            .expect(&format!(
                "Error: Sequence bits is too large '{}'", sequence_bits))
            .checked_sub(node_id_bits)
            .expect(&format!(
                "Error: Sum of bits is too large, maximum value 64. Node ID bits '{}', Sequence bits '{}'",
                node_id_bits, sequence_bits
            ))
            .checked_sub(sign_bits)
            .expect(&format!(
                "Error: Sum of bits is too large, maximum value 64. Sign bits '{}', Sequence bits '{}', Node ID bits '{}'", 
                sign_bits, sequence_bits, node_id_bits
            ));
        SequenceProperties {
            custom_epoch,
            timestamp_bits,
            node_id_bits,
            sequence_bits,
            current_timestamp: None,
            last_timestamp: None,
            micros_ten_power,
            node_id,
            sign_bits,
            sequence: 0,
            max_sequence: (2 as u16).pow(sequence_bits.into()),
            backoff_cooldown_start_ns,
        }
    }
}

pub fn generate_id(properties: &mut SequenceProperties) -> u64 {
    properties.last_timestamp = properties.current_timestamp;
    properties.current_timestamp = Some(timestamp_from_custom_epoch(
        properties.custom_epoch,
        properties.micros_ten_power,
    ));
    if let Some(last_timestamp) = properties.last_timestamp {
        let current_timestamp = properties.current_timestamp.unwrap();
        if current_timestamp < last_timestamp {
            println!("Error: System Clock moved backwards. Current timestamp '{}' is earlier than last registered '{}'.", 
                current_timestamp, last_timestamp);
            if properties.sequence == properties.max_sequence {
                wait_next_timestamp(
                    last_timestamp,
                    properties.custom_epoch,
                    properties.micros_ten_power,
                    properties.backoff_cooldown_start_ns,
                );
                // After timestamp changed reset to start a new sequence
                properties.sequence = 0;
            } else {
                wait_until_last_timestamp(
                    last_timestamp,
                    properties.custom_epoch,
                    properties.micros_ten_power,
                    properties.backoff_cooldown_start_ns,
                );
            }
            properties.current_timestamp = Some(timestamp_from_custom_epoch(
                properties.custom_epoch,
                properties.micros_ten_power,
            ));
        } else if properties.current_timestamp.unwrap() != last_timestamp {
            properties.sequence = 0;
        }
    }
    let new_id = to_id(properties);
    properties.sequence += 1;
    if properties.sequence == properties.max_sequence {
        wait_next_timestamp(
            properties.last_timestamp.unwrap(),
            properties.custom_epoch,
            properties.micros_ten_power,
            properties.backoff_cooldown_start_ns,
        );
        properties.current_timestamp = Some(timestamp_from_custom_epoch(
            properties.custom_epoch,
            properties.micros_ten_power,
        ));
        // After timestamp changed reset to start a new sequence
        properties.sequence = 0;
    }
    new_id
}

pub fn wait_next_timestamp(
    last_timestamp: u64,
    custom_epoch: SystemTime,
    micros_ten_power: u8,
    backoff_cooldown_start_ns: u64,
) {
    let mut current_timestamp = timestamp_from_custom_epoch(custom_epoch, micros_ten_power);
    let backoff_cooldown_ns: u64 = backoff_cooldown_start_ns;
    while current_timestamp <= last_timestamp {
        sleep(Duration::from_nanos(backoff_cooldown_ns));
        current_timestamp = timestamp_from_custom_epoch(custom_epoch, micros_ten_power);
        // Double the cooldown wait period (exponential backoff)
        backoff_cooldown_ns
            .checked_add(backoff_cooldown_ns)
            .expect(&format!(
                "Error: Cannot double backoff cooldown, maximum value reached '{}'",
                backoff_cooldown_ns
            ));
    }
}

pub fn wait_until_last_timestamp(
    last_timestamp: u64,
    custom_epoch: SystemTime,
    micros_ten_power: u8,
    backoff_cooldown_start_ns: u64,
) {
    let mut current_timestamp = timestamp_from_custom_epoch(custom_epoch, micros_ten_power);
    let backoff_cooldown_ns: u64 = backoff_cooldown_start_ns;
    while current_timestamp < last_timestamp {
        sleep(Duration::from_nanos(backoff_cooldown_ns));
        current_timestamp = timestamp_from_custom_epoch(custom_epoch, micros_ten_power);
        // Double the cooldown wait period (exponential backoff)
        backoff_cooldown_ns
            .checked_add(backoff_cooldown_ns)
            .expect(&format!(
                "Error: Cannot double backoff cooldown, maximum value reached '{}'",
                backoff_cooldown_ns
            ));
    }
}

pub fn to_id(properties: &mut SequenceProperties) -> u64 {
    let timestamp_shift_bits = properties.node_id_bits + properties.sequence_bits;
    let sequence_shift_bits = properties.sequence_bits;
    let mut id = properties.current_timestamp.unwrap() << (timestamp_shift_bits);
    id |= (properties.sequence << sequence_shift_bits) as u64;
    id |= properties.node_id as u64;
    id
}

fn main() {
    let mut args = Opt::from_args();
    let dotenv_file = &args.dotenv_file;
    if Path::new(dotenv_file).exists() {
        dotenv::from_filename(dotenv_file).expect(&format!(
            "Error: Could not retrieve environment variables from configuration file '{}'",
            dotenv_file
        ));
        for (key, value) in env::vars() {
            if key == "CUSTOM_EPOCH" && value != "" {
                args.custom_epoch = value.parse::<String>().expect(&format!(
                    "Error: Couldn't parse value CUSTOM_EPOCH '{}' as String, invalid UTF-8 characters", value)
                );
            }
            if key == "NODE_ID_BITS" && value != "" {
                args.node_id_bits = value.parse::<u8>().expect(&format!(
                    "Error: NODE_ID_BITS '{}' couldn't be interpreted as value between 0 and 255",
                    value
                ));
            }
            if key == "SEQUENCE_BITS" && value != "" {
                args.sequence_bits = value.parse::<u8>().expect(&format!(
                    "Error: SEQUENCE_BITS '{}' couldn't be interpreted as value between 0 and 255",
                    value
                ));
            }
            if key == "MICROS_TEN_POWER" && value != "" {
                args.micros_ten_power = value.parse::<u8>().expect(&format!(
                    "Error: MICROS_TEN_POWER '{}' couldn't be interpreted as value between 0 and 255", value)
                );
            }
            if key == "SIGN_BITS" && value != "" {
                args.sign_bits = value.parse::<u8>().expect(&format!(
                    "Error: SIGN_BITS '{}' couldn't be interpreted as value between 0 and 255",
                    value
                ));
            }
            if key == "COOLDOWN_NS" && value != "" {
                args.cooldown_ns = value.parse::<u64>().expect(&format!(
                    "Error: COOLDOWN_NS '{}' couldn't be interpreted as an unsigned integer value",
                    value
                ));
            }
        }
    }

    let custom_epoch_millis = DateTime::parse_from_rfc3339(&args.custom_epoch)
        .expect(&format!(
            "Error: Could not parse CUSTOM_EPOCH '{}' as an RFC-3339/ISO-8601 datetime.",
            args.custom_epoch
        ))
        .timestamp_millis();
    let custom_epoch = UNIX_EPOCH
        .checked_add(Duration::from_millis(custom_epoch_millis as u64))
        .expect(&format!(
            "Error: Could not generate a SystemTime custom epoch from milliseconds timestamp '{}'",
            custom_epoch_millis
        ));
    let mut properties = SequenceProperties::new(
        custom_epoch,
        args.node_id_bits,
        args.node_id,
        args.sequence_bits,
        args.micros_ten_power,
        args.sign_bits,
        args.cooldown_ns,
    );
    let mut vector_ids: Vec<u64> = vec![0; args.number];
    if args.debug {
        let time_now = SystemTime::now();
        for element in vector_ids.iter_mut() {
            *element = generate_id(&mut properties);
        }
        let elapsed = time_now
            .elapsed()
            .expect("Error: Failed to get elapsed time.")
            .as_nanos();
        for (index, element) in vector_ids.into_iter().enumerate() {
            println!("{}: {}", index, element);
        }
        println!("It took {:?} nanoseconds", elapsed);
    } else {
        for (index, element) in vector_ids.iter_mut().enumerate() {
            *element = generate_id(&mut properties);
            println!("{}: {}", index, element);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn timestamp_from_custom_epoch() {
        // Perform consistency tests for datetime calculation from a custom epoch
        // First case: Compare system time against custom epoch set to UNIX_EPOCH
        // Second case: Set CUSTOM_EPOCH to test start time and compare timestamp
        // calculation against known sleep duration interval
        use super::*;
        let time_now = SystemTime::now();
        let millis_start = time_now
            .duration_since(UNIX_EPOCH)
            .expect("Error: Failed to get current time as duration from epoch.")
            .as_millis();
        sleep(Duration::from_millis(50));
        // Test UNIX EPOCH
        let millis_after = timestamp_from_custom_epoch(UNIX_EPOCH, 3);
        // More than expected 50ms. Upper boundary cannot be ascertained as Normal distribution
        // CPU low-power states and/or older hardware can cause signifficant differences.
        // (although rather then a Normal distribution, it is instead the case that a Pareto
        // distribution applies, making it impossible to set high enough value for the test
        // not to fail on ocassion)
        let substracted_times = millis_after.checked_sub(millis_start as u64).unwrap();
        println!("Too small time difference between times calculated\nfrom UNIX_EPOCH using independent functions.\n\nEpoch System Time - Time Difference w/Epoch = {} ms,\nexpected greater or equals than sleep interval 50 ms.\n", substracted_times);
        assert!(substracted_times < 50);
        // If too big upper boundary there could be numerical errors.
        assert!((millis_after.checked_sub(millis_start as u64).unwrap()) < 90);
        // Test a CUSTOM EPOCH in tenths of a millisecond
        let custom_epoch = UNIX_EPOCH
            .checked_add(Duration::from_millis(millis_start as u64))
            .expect("Error: Failed to create custom epoch.");
        let tenths_millis_custom_epoch_time = timestamp_from_custom_epoch(custom_epoch, 2);
        // Wait a bit to prevent Option to call unwrap() on None below
        // If both timestamps are within small margin substraction of u64
        // can result in 'panicked at attempt to subtract with overflow'
        // and checked_sub returns None value
        sleep(Duration::from_millis(2));
        // convert elapsed time from microseconds into tenths of a millisecond (0,1ms = 100 mcs)
        let ten: u64 = 10;
        let power_two: u32 = 2;
        let tenths_millis_elapsed_time = (time_now
            .elapsed()
            .expect("Error: Failed to get elapsed time.")
            .as_micros() as u64)
            / ten.pow(power_two);
        let substracted_times = tenths_millis_elapsed_time
            .checked_sub(tenths_millis_custom_epoch_time)
            .unwrap();
        println!("Too high time difference between calculated time from\nCustom Epoch set at test start and actual elapsed\ntime since the test started.\n\nElapsed Time - Calculated Time Custom Epoch = {} mcs,\nexpected under 100 mcs\n\nPlease note that Pareto distribution applies and it\nis impossible to ensure a high enough difference for\nthe test not to fail on ocassion.\n\nReview only after ensuring repeated failures.\n", substracted_times);
        println!("substracted_times: {}", substracted_times);
        // Substract custom epoch result with Rust's own elapsed time
        // Upper boundary uncertainty set up high at 100mcs more than expected 511mcs as exponential
        // distribution, CPU low-power states and/or older hardware can cause signifficant differences.
        assert!(substracted_times < 100);
    }

    #[test]
    fn wait_until_last_timestamp() {
        // Case where system clock is readjusted 50ms into the past
        // Current sequence wouldn't be exhausted but script cools down
        // until at least matching the previously stored timestamp.
        use super::*;
        let calculated_time_after_50ms: u64 = SystemTime::now()
            .checked_add(Duration::from_millis(50))
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .expect("Error: Failed to get duration from epoch of timestamp 50ms into the future.")
            .as_millis() as u64;
        // Function itself serves as an sleep call if correct
        wait_until_last_timestamp(calculated_time_after_50ms, UNIX_EPOCH, 3, 1500);
        // Wait a bit to prevent Option to call unwrap() on None below
        // If both timestamps are within small margin substraction of u64
        // can result in 'panicked at attempt to subtract with overflow'
        // and checked_sub returns None value.
        // Furthermore: It could also result in useless assert comparing
        // if an unsigned integer is higher or equal to zero
        sleep(Duration::from_millis(1));
        let time_after_50ms: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Error: Failed to get current time as duration from epoch.")
            .as_millis() as u64;
        let substracted_times = time_after_50ms
            .checked_sub(calculated_time_after_50ms)
            .unwrap();
        assert!(substracted_times > 0);
        println!("Too high time difference while waiting for last timestamp\nafter clock moved backwards\n\nTime Calculated - Actual Time = {} ms, expected under 25 ms\n\nPlease note that Pareto distribution applies and it\nis impossible to ensure a high enough difference for\nthe test not to fail on ocassion.\n\nReview only after ensuring repeated failures.\n", substracted_times);
        // Assert an upper boundary to how high of a difference there can be.
        // If implementation is correct, the timestampts should be within few
        // ms of one another according to a Normal distribution in recent
        // hardware and normal CPU priority (although rather a Pareto
        // distribution applies, making it impossible to set a value high
        // enough for the test not to fail on ocassion)
        assert!(substracted_times < 25);
    }
    #[test]
    fn wait_next_timestamp() {
        // Case where sequence would be exhausted and for that reason
        // script cools down until at least there exists a difference
        // between the current system time and the last known timestamp.
        use super::*;
        let calculated_time_after_10ms: u64 = SystemTime::now()
            .checked_add(Duration::from_millis(10))
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .expect("Error: Failed to get duration from epoch of timestamp 50ms into the future.")
            .as_millis() as u64;
        // Function itself serves as an sleep call if correct
        wait_next_timestamp(calculated_time_after_10ms, UNIX_EPOCH, 3, 1500);
        let time_after_11ms: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Error: Failed to get current time as duration from epoch.")
            .as_millis() as u64;
        let substracted_times = time_after_11ms
            .checked_sub(calculated_time_after_10ms)
            .unwrap();
        assert!(substracted_times > 0);
        println!("Too high time difference while waiting for next timestamp\n\nNext timestamp - Last Timestamp = {} ms, expected under 25 ms\n\nPlease note that Pareto distribution applies and it\nis impossible to ensure a high enough difference for\nthe test not to fail on ocassion.\n\nReview only after ensuring repeated failures.\n", substracted_times);
        // Assert an upper boundary to how high of a difference there can be.
        // If implementation is correct, the timestampts should be within few
        // ms of one another according to a Normal distribution in recent
        // hardware and normal CPU priority (although rather a Pareto
        // distribution applies, making it impossible to set a value high
        // enough for the test not to fail on ocassion)
        assert!(substracted_times < 25);
    }
}
