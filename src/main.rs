use ::sequence_generator::*;
use chrono::DateTime;
use std::env;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(default_value = "1", long, short)]
    number: usize,
    #[structopt(default_value = "2020-01-01T00:00:00Z", long, short)]
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
    unused_bits: u8,
    #[structopt(default_value = ".env", long)]
    dotenv_file: String,
    #[structopt(default_value = "1500", long)]
    cooldown_ns: u64,
    #[structopt(long, short)]
    debug: bool,
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
                    "Error: NODE_ID_BITS '{}' couldn't be interpreted as value between 0 and 64",
                    value
                ));
            }
            if key == "SEQUENCE_BITS" && value != "" {
                args.sequence_bits = value.parse::<u8>().expect(&format!(
                    "Error: SEQUENCE_BITS '{}' couldn't be interpreted as value between 0 and 64",
                    value
                ));
            }
            if key == "MICROS_TEN_POWER" && value != "" {
                args.micros_ten_power = value.parse::<u8>().expect(&format!(
                    "Error: MICROS_TEN_POWER '{}' couldn't be interpreted as value between 0 and 64", value)
                );
            }
            if key == "UNUSED_BITS" && value != "" {
                args.unused_bits = value.parse::<u8>().expect(&format!(
                    "Error: UNUSED_BITS '{}' couldn't be interpreted as value between 0 and 64",
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
    let mut properties = sequence_generator::SequenceProperties::new(
        custom_epoch,
        args.node_id_bits,
        args.node_id,
        args.sequence_bits,
        args.micros_ten_power,
        args.unused_bits,
        args.cooldown_ns,
    );
    let mut vector_ids: Vec<u64> = vec![0; args.number];
    if args.debug {
        let time_now = SystemTime::now();
        for element in vector_ids.iter_mut() {
            *element = sequence_generator::generate_id(&mut properties).unwrap_or_else(
                |error| {
                    panic!(format!(
                        "SequenceGeneratorError: Failed to get ID from properties {:?}. SystemTimeError difference {:?}",
                        properties,
                        (error).duration()
                    ))
                }
            );
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
            *element = sequence_generator::generate_id(&mut properties).unwrap_or_else(
                |error| {
                    panic!(format!(
                        "SequenceGeneratorError: Failed to get ID from properties {:?}. SystemTimeError difference {:?}",
                        properties,
                        (error).duration()
                    ))
                });
            println!("{}: {}", index, element);
        }
    }
}
