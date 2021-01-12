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
    ) -> Self {
        let sign_bits = 1;
        let timestamp_bits = (64 as u8)
            .checked_sub(sign_bits)
            .unwrap()
            .checked_sub(node_id_bits)
            .unwrap()
            .checked_sub(sequence_bits)
            .unwrap();
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
    if properties.current_timestamp.unwrap() < properties.last_timestamp.unwrap() {
        panic!(
            "Error: Invalid System Clock state. New timestamp is earlier than previously registered timestamp."
        )
    }
    let new_id = to_id(properties);
    properties.sequence += 1;
    if properties.sequence == properties.max_sequence {
        if properties.sequence == properties.max_sequence {}
    } else {
        // if timestamp changed reset to start a new sequence
        properties.sequence = 0;
    }
    new_id
}

pub fn to_id(properties: &mut SequenceProperties) -> u64 {
    let mut id = properties.current_timestamp.unwrap()
        << (properties.node_id_bits + properties.sequence_bits);
    id |= (properties.node_id << properties.sequence_bits) as u64;
    id |= properties.sequence_bits as u64;
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
                args.custom_epoch = value.clone();
                println!("CUSTOM_EPOCH: {}", args.custom_epoch)
            }
            if key == "NODE_ID_BITS" && value != "" {
                args.node_id_bits = value.parse::<u8>().expect(
                    "Error: NODE_ID_BITS couldn't be interpreted as value between 0 and 255",
                );
                println!("NODE_ID_BITS: {}", args.node_id_bits)
            }
            if key == "SEQUENCE_BITS" && value != "" {
                args.sequence_bits = value.parse::<u8>().expect(
                    "Error: SEQUENCE_BITS couldn't be interpreted as value between 0 and 255",
                );
                println!("SEQUENCE_BITS: {}", args.sequence_bits)
            }
            if key == "MICROS_TEN_POWER" && value != "" {
                args.micros_ten_power = value.parse::<u8>().expect(
                    "Error: MICROS_TEN_POWER couldn't be interpreted as value between 0 and 255",
                );
                println!("MICROS_TEN_POWER: {}", args.micros_ten_power)
            }
        }
    }

    let mut properties = SequenceProperties::new(
        UNIX_EPOCH,
        args.node_id_bits,
        args.node_id,
        args.sequence_bits,
        args.micros_ten_power,
    );
    let time_now = SystemTime::now();
    generate_id(&mut properties);
    println!(
        "It took {:?} nanoseconds",
        time_now
            .elapsed()
            .expect("Error: Failed to get elapsed time.")
            .as_nanos()
    );
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
            .as_millis();
        sleep(Duration::from_millis(50));
        // Test UNIX EPOCH
        let millis_after = timestamp_from_custom_epoch(UNIX_EPOCH, 3);
        println!(
            "millis_after {:?}, millis_start {:?}",
            millis_after, millis_start
        );
        println!(
            "millis_after - millis_start = {:?}",
            millis_after.checked_sub(millis_start as u64)
        );
        // 25ms more than expected 50ms as exponential distribution, CPU low-power
        // states and/or older hardware can cause signifficant differences.
        assert!((millis_after.checked_sub(millis_start as u64).unwrap()) < 75);
        // Test a CUSTOM EPOCH since launching test
        let custom_epoch = UNIX_EPOCH
            .checked_add(Duration::from_millis(millis_start as u64))
            .expect("Error: Failed to create custom epoch.");
        let millis_custom_epoch_time = timestamp_from_custom_epoch(custom_epoch, 3);
        // Guarantee difference in millis
        sleep(Duration::from_millis(1));
        let millis_elapsed_time = time_now
            .elapsed()
            .expect("Error: Failed to get elapsed time.")
            .as_millis() as u64;
        println!(
            "millis_elapsed_time {:?}, millis_custom_epoch_time {:?}",
            millis_elapsed_time, millis_custom_epoch_time
        );
        println!(
            "millis_elapsed_time - millis_custom_epoch_time = {:?}",
            millis_elapsed_time.checked_sub(millis_custom_epoch_time)
        );
        // 25ms more than expected 1ms as exponential distribution, CPU low-power
        // states and/or older hardware can cause signifficant differences.
        assert!(
            (millis_elapsed_time
                .checked_sub(millis_custom_epoch_time)
                .unwrap())
                < 26
        );
    }
}
