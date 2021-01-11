use dotenv::dotenv;
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

pub fn millis_from_custom_epoch(custom_epoch: SystemTime) -> u64 {
    SystemTime::now()
        .duration_since(custom_epoch)
        .expect("Error: Failed to create Timestamp from custom epoch.")
        .as_millis() as u64
}

pub struct SequenceGenerator {
    sign_bits: u8,
    timestamp_bits: u8,
    pub node_id_bits: u8,
    pub sequence_bits: u8,
    pub custom_epoch: SystemTime,
    last_millis: Option<u64>,
    pub node_id: u16,
}

impl SequenceGenerator {
    pub fn new(
        custom_epoch: SystemTime,
        node_id_bits: u8,
        node_id: u16,
        sequence_bits: u8,
    ) -> Self {
        let timestamp_bits = 64 - 1 - node_id_bits - sequence_bits;
        SequenceGenerator {
            custom_epoch,
            timestamp_bits,
            node_id_bits,
            sequence_bits,
            last_millis: None,
            node_id,
            sign_bits: 1,
        }
    }
    pub fn generate_id(
        &mut self,
        custom_epoch: SystemTime,
        node_id: u16,
        node_id_bits: u8,
        sequence_bits: u8,
        debug: bool,
    ) {
        unimplemented!()
    }
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
        }
    }

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
