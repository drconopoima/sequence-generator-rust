use ::sequence_generator::*;
use std::convert::TryFrom;
use std::env;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use structopt::StructOpt;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(default_value = "1", long, short)]
    number: usize,
    #[structopt(long, short)]
    custom_epoch: Option<String>,
    #[structopt(long)]
    micros_ten_power: Option<u8>,
    #[structopt(long)]
    node_id_bits: Option<u8>,
    #[structopt(long)]
    sequence_bits: Option<u8>,
    #[structopt(long)]
    node_id: Option<u16>,
    #[structopt(long)]
    unused_bits: Option<u8>,
    #[structopt(default_value = ".env", long)]
    dotenv_file: String,
    #[structopt(long)]
    cooldown_ns: Option<u64>,
    #[structopt(long, short)]
    debug: bool,
}

fn main() {
    let mut args = Opt::from_args();
    let dotenv_file = &args.dotenv_file;
    if Path::new(dotenv_file).exists() {
        dotenv::from_filename(dotenv_file).unwrap_or_else(|_| {
            panic!(
                "Error: Could not retrieve environment variables from configuration file '{}'",
                dotenv_file
            )
        });
        for (key, value) in env::vars() {
            if key == "CUSTOM_EPOCH" && !value.is_empty() && args.custom_epoch.is_none() {
                args.custom_epoch = Some(value.parse::<String>().unwrap_or_else(|_| {panic!(
                    "Error: Couldn't parse value CUSTOM_EPOCH '{}' as String, invalid UTF-8 characters", value)
                }));
            }
            if key == "NODE_ID_BITS" && !value.is_empty() && args.node_id_bits.is_none() {
                args.node_id_bits = Some(value.parse::<u8>().unwrap_or_else(|_| {
                    panic!(
                    "Error: NODE_ID_BITS '{}' couldn't be interpreted as value between 0 and 64",
                    value
                )
                }));
            }
            if key == "SEQUENCE_BITS" && !value.is_empty() && args.sequence_bits.is_none() {
                args.sequence_bits = Some(value.parse::<u8>().unwrap_or_else(|_| {
                    panic!(
                    "Error: SEQUENCE_BITS '{}' couldn't be interpreted as value between 0 and 64",
                    value
                )
                }));
            }
            if key == "MICROS_TEN_POWER" && !value.is_empty() && args.micros_ten_power.is_none() {
                args.micros_ten_power = Some(value.parse::<u8>().unwrap_or_else(|_| {panic!(
                    "Error: MICROS_TEN_POWER '{}' couldn't be interpreted as value between 0 and 64", value)
                }));
            }
            if key == "UNUSED_BITS" && !value.is_empty() && args.unused_bits.is_none() {
                args.unused_bits = Some(value.parse::<u8>().unwrap_or_else(|_| {
                    panic!(
                        "Error: UNUSED_BITS '{}' couldn't be interpreted as value between 0 and 64",
                        value
                    )
                }));
            }
            if key == "COOLDOWN_NS" && !value.is_empty() && args.cooldown_ns.is_none() {
                args.cooldown_ns = Some(value.parse::<u64>().unwrap_or_else(|_| {
                    panic!(
                    "Error: COOLDOWN_NS '{}' couldn't be interpreted as an unsigned integer value",
                    value
                )
                }));
            }
        }
    }
    if args.custom_epoch.is_none() {
        args.custom_epoch = Some("2020-01-01T00:00:00Z".to_owned());
    }

    if args.micros_ten_power.is_none() {
        args.micros_ten_power = Some(2_u8);
    }

    if args.node_id_bits.is_none() {
        args.node_id_bits = Some(9_u8);
    }

    if args.sequence_bits.is_none() {
        args.sequence_bits = Some(11_u8);
    }

    if args.node_id.is_none() {
        args.node_id = Some(0_u16);
    }

    if args.unused_bits.is_none() {
        args.unused_bits = Some(0_u8);
    }

    if args.cooldown_ns.is_none() {
        args.cooldown_ns = Some(1500_u64);
    }

    let custom_epoch_millis_i128 =
        OffsetDateTime::parse(args.custom_epoch.as_ref().unwrap(), &Rfc3339)
            .unwrap_or_else(|_| {
                panic!(
                    "Error: Could not parse CUSTOM_EPOCH '{}' as an RFC-3339/ISO-8601 datetime.",
                    args.custom_epoch.as_ref().unwrap()
                )
            })
            .unix_timestamp_nanos()
            / 1000000;
    let custom_epoch_millis = i64::try_from(custom_epoch_millis_i128).unwrap();
    let custom_epoch = UNIX_EPOCH
        .checked_add(Duration::from_millis(custom_epoch_millis as u64))
        .unwrap_or_else(|| {
            panic!(
            "Error: Could not generate a SystemTime custom epoch from milliseconds timestamp '{}'",
            custom_epoch_millis
        )
        });
    let properties = Rc::new(sequence_generator::SequenceProperties::new(
        custom_epoch,
        args.node_id_bits.unwrap(),
        args.node_id.unwrap(),
        args.sequence_bits.unwrap(),
        args.micros_ten_power.unwrap(),
        args.unused_bits.unwrap(),
        args.cooldown_ns.unwrap(),
    ));
    let mut vector_ids: Vec<u64> = vec![0; args.number];
    if args.debug {
        let time_now = SystemTime::now();
        for element in vector_ids.iter_mut() {
            *element = sequence_generator::generate_id(&properties).unwrap_or_else(
                |error| {
                    panic!(
                        "SequenceGeneratorError: Failed to get ID from properties {:?}. SystemTimeError difference {:?}",
                        properties,
                        (error).duration()
                    )
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
            *element = sequence_generator::generate_id(&properties).unwrap_or_else(
                |error| {
                    panic!(
                        "SequenceGeneratorError: Failed to get ID from properties {:?}. SystemTimeError difference {:?}",
                        properties,
                        (error).duration()
                    )
                });
            println!("{}: {}", index, element);
        }
    }
}
