use dotenv;
use std::env;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, short)]
    number: usize,
    #[structopt(long, short)]
    custom_epoch: String,
    #[structopt(default_value = "3", long)]
    micros_ten_power: u8,
    #[structopt(default_value = "10", long)]
    node_id_bits: u8,
    #[structopt(default_value = "12", long)]
    sequence_bits: u8,
    #[structopt(default_value = "0", long)]
    node_id: u16,
    #[structopt(default_value = "0", long)]
    sign_bits: u8,
    #[structopt(default_value = ".env", long)]
    dotenv_file: String,
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
    sign_bits: u8,
    timestamp_bits: u8,
    pub node_id_bits: u8,
    pub sequence_bits: u8,
    pub custom_epoch: SystemTime,
    current_timestamp: Option<u64>,
    last_timestamp: Option<u64>,
    pub micros_ten_power: u8,
    pub node_id: u16,
    pub sequence: u16,
    max_sequence: u16,
}

impl SequenceProperties {
    pub fn new(
        custom_epoch: SystemTime,
        node_id_bits: u8,
        node_id: u16,
        sequence_bits: u8,
        micros_ten_power: u8,
        sign_bits: u8,
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
        if properties.current_timestamp.unwrap() < last_timestamp {
            panic!(
                    "Error: System Clock moved backwards. New timestamp is earlier than previously registered."
                )
        }
    }
    let new_id = to_id(properties);
    properties.sequence += 1;
    if properties.sequence == properties.max_sequence {
        if properties.sequence == properties.max_sequence {
            // wait_next_timestamp()
        }
    } else {
        // if timestamp changed reset to start a new sequence
        properties.sequence = 0;
    }
    new_id
}

// pub fn wait_next_timestamp() {}

pub fn to_id(properties: &mut SequenceProperties) -> u64 {
    let timestamp_shift_bits = properties.node_id_bits + properties.sequence_bits;
    let node_id_shift_bits = properties.sequence_bits;
    let mut id = properties.current_timestamp.unwrap() << (timestamp_shift_bits);
    id |= (properties.node_id << node_id_shift_bits) as u64;
    id |= properties.sequence as u64;
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
        }
    }

    let mut properties = SequenceProperties::new(
        UNIX_EPOCH,
        args.node_id_bits,
        args.node_id,
        args.sequence_bits,
        args.micros_ten_power,
        args.sign_bits,
    );
    let time_now = SystemTime::now();
    let next_id = generate_id(&mut properties);
    let elapsed = time_now
        .elapsed()
        .expect("Error: Failed to get elapsed time.")
        .as_nanos();
    println!("The id generated is: '{}'", next_id);
    println!("It took {:?} nanoseconds", elapsed);
}

#[cfg(test)]
mod tests {
    #[test]
    fn timestamp_from_custom_epoch() {
        use super::*;
        use std::thread::sleep;
        use std::time::Duration;
        let time_now = SystemTime::now();
        let millis_start = time_now
            .duration_since(UNIX_EPOCH)
            .expect("Error: Failed to get current time as duration from epoch.")
            .as_millis();
        sleep(Duration::from_millis(50));
        // Test UNIX EPOCH
        let millis_after = timestamp_from_custom_epoch(UNIX_EPOCH, 3);
        // 25ms more than expected 50ms as exponential distribution, CPU low-power
        // states and/or older hardware can cause signifficant differences.
        assert!((millis_after.checked_sub(millis_start as u64).unwrap()) < 75);
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
        // Substract custom epoch result with Rust's own elapsed time
        // 150mcs more than expected 511mcs as exponential distribution, CPU low-power
        // states and/or older hardware can cause signifficant differences.
        assert!(
            (tenths_millis_elapsed_time
                .checked_sub(tenths_millis_custom_epoch_time)
                .unwrap())
                < 150
        );
    }
}
