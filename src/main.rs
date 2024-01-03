use ::sequence_generator::*;
use std::convert::TryFrom;
use std::env;
use std::path::Path;
use std::process;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use structopt::StructOpt;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(
        short = "-n",
        long = "--number",
        help = "Amount/Quantity of sequence values requested. [Default: 1]"
    )]
    number: Option<usize>,
    #[structopt(
        short = "-q",
        long = "--quantity",
        help = "Amount/Quantity of sequence values requested. [Default: 1]"
    )]
    quantity: Option<usize>,
    #[structopt(
        short = "-c",
        long = "--custom-epoch",
        help = "Custom epoch in RFC3339 format. [Default: '2020-01-01T00:00:00Z' i.e. Jan 01 2020 00:00:00 UTC]",
        env = "CUSTOM_EPOCH"
    )]
    custom_epoch: Option<String>,
    #[structopt(
        short = "-m",
        long = "--micros-ten-power",
        help = "Exponent multiplier base 10 in microseconds for timestamp. [Default: 2 (operate in tenths of milliseconds)]",
        env = "MICROS_TEN_POWER"
    )]
    micros_ten_power: Option<u8>,
    #[structopt(
        short = "-w",
        long = "--node-id-bits",
        help = "Bits used for storing worker and datacenter information. [Default: 9 (range: 0-511). Maximum: 16. Minimum: 1]",
        env = "NODE_ID_BITS"
    )]
    node_id_bits: Option<u8>,
    #[structopt(
        short = "-s",
        long = "--sequence-bits",
        help = "Bits used for contiguous sequence values. [Default: 11 (range: 0-2047). Maximum: 16. Minimum: 1]",
        env = "SEQUENCE_BITS"
    )]
    sequence_bits: Option<u8>,
    #[structopt(
        short = "-i",
        long = "--node-id",
        help = "Numerical identifier for worker and datacenter information. [Default: 0]",
        env = "NODE_ID"
    )]
    node_id: Option<u16>,
    #[structopt(
        short = "-u",
        long = "--unused-bits",
        help = "Unused (sign) bits at the left-most of the sequence ID. [Default: 0. Maximum: 7]",
        env = "UNUSED_BITS"
    )]
    unused_bits: Option<u8>,
    #[structopt(
        long = "--sign-bits",
        help = "Unused (sign) bits at the left-most of the sequence ID. [Default: 0. Maximum: 8]",
        env = "SIGN_BITS"
    )]
    sign_bits: Option<u8>,
    #[structopt(
        default_value = ".env",
        long = "--dotenv-file",
        help = "File for configuration variables. [Default: '${pwd}/.env']"
    )]
    dotenv_file: String,
    #[structopt(
        long = "--cooldown-ns",
        help = "Initial time in nanoseconds for exponential backoff wait after sequence is exhausted. [Default: 1000]",
        env = "COOLDOWN_NS"
    )]
    cooldown_ns: Option<u64>,
    #[structopt(short = "-d", long = "--debug")]
    debug: bool,
}

fn main() {
    let mut args = Opt::from_args();
    let dotenv_file = &args.dotenv_file;
    if Path::new(dotenv_file).exists() {
        dotenv::from_filename(dotenv_file).unwrap_or_else(|_| {
            panic!(
                "ERROR: Could not retrieve environment variables from configuration file '{}'",
                dotenv_file
            )
        });
        for (key, value) in env::vars() {
            if key == "CUSTOM_EPOCH" && !value.is_empty() && args.custom_epoch.is_none() {
                args.custom_epoch = Some(value.parse::<String>().unwrap_or_else(|_| {panic!(
                    "ERROR: Couldn't parse value CUSTOM_EPOCH '{}' as String, invalid UTF-8 characters", value)
                }));
            }
            if key == "NODE_ID_BITS" && !value.is_empty() && args.node_id_bits.is_none() {
                args.node_id_bits = Some(value.parse::<u8>().unwrap_or_else(|_| {
                    panic!(
                    "ERROR: NODE_ID_BITS '{}' couldn't be interpreted as value between 1 and 16",
                    value
                )
                }));
            }

            if key == "SEQUENCE_BITS" && !value.is_empty() && args.sequence_bits.is_none() {
                args.sequence_bits = Some(value.parse::<u8>().unwrap_or_else(|_| {
                    panic!(
                    "ERROR: SEQUENCE_BITS '{}' couldn't be interpreted as value between 1 and 16",
                    value
                )
                }));
            }
            if key == "MICROS_TEN_POWER" && !value.is_empty() && args.micros_ten_power.is_none() {
                args.micros_ten_power = Some(value.parse::<u8>().unwrap_or_else(|_| {panic!(
                    "ERROR: MICROS_TEN_POWER '{}' couldn't be interpreted as value between 0 and 64", value)
                }));
            }
            if key == "UNUSED_BITS" && !value.is_empty() && args.unused_bits.is_none() {
                args.unused_bits = Some(value.parse::<u8>().unwrap_or_else(|_| {
                    panic!(
                        "ERROR: UNUSED_BITS '{}' couldn't be interpreted as value between 0 and 7",
                        value
                    )
                }));
            }
            if key == "SIGN_BITS" && !value.is_empty() && args.unused_bits.is_none() {
                args.unused_bits = Some(value.parse::<u8>().unwrap_or_else(|_| {
                    panic!(
                        "ERROR: SIGN_BITS '{}' couldn't be interpreted as value between 0 and 7",
                        value
                    )
                }));
            }
            if key == "COOLDOWN_NS" && !value.is_empty() && args.cooldown_ns.is_none() {
                args.cooldown_ns = Some(value.parse::<u64>().unwrap_or_else(|_| {
                    panic!(
                    "ERROR: COOLDOWN_NS '{}' couldn't be interpreted as an unsigned integer value",
                    value
                )
                }));
            }
        }
    }
    if args.quantity.is_some() && args.number.is_some() {
        panic!(
            "ERROR: Conflicting parameters. Must only specify one of either '--quantity,-q' or '--number,-n'"
        )
    }
    if args.sign_bits.is_some() && args.unused_bits.is_some() {
        panic!(
            "ERROR: Conflicting parameters. Must only specify one of either '--unused-bits,-u' or '--sign-bits'"
        )
    }
    if let Some(value) = args.sign_bits {
        if value > 7 {
            panic!(
                "ERROR: SIGN_BITS '{}' is larger than the maximum value of 7.",
                value
            )
        }
    };
    if args.sign_bits.is_some() {
        args.unused_bits = args.sign_bits;
    }
    if let Some(value) = args.unused_bits {
        if value > 7 {
            panic!(
                "ERROR: UNUSED_BITS '{}' is larger than the maximum value of 7.",
                value
            )
        }
    };
    if let Some(value) = args.sequence_bits {
        if value > 16 {
            panic!(
                "ERROR: SEQUENCE_BITS '{}' is larger than the maximum value of 16.",
                value
            )
        }
        if value == 0 {
            panic!(
                "ERROR: SEQUENCE_BITS '{}' must be larger or equal than 1.",
                value
            )
        }
    };
    if let Some(value) = args.node_id_bits {
        if value > 16 {
            panic!(
                "ERROR: NODE_ID_BITS '{}' is larger than the maximum value of 16.",
                value
            )
        }
        if value == 0 {
            panic!(
                "ERROR: NODE_ID_BITS '{}' must be larger or equal than 1.",
                value
            )
        }
    };
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
        args.cooldown_ns = Some(1000_u64);
    }

    if args.number.is_none() {
        if let Some(value) = args.quantity {
            args.number = Some(value)
        } else {
            args.number = Some(1_usize)
        }
    }
    if let Some(value) = args.number {
        if value == 0 {
            println!("WARNING: No ids were requested. Exiting.");
            process::exit(0x0100);
        }
    }
    let custom_epoch_millis_i128 =
        OffsetDateTime::parse(args.custom_epoch.as_ref().unwrap(), &Rfc3339)
            .unwrap_or_else(|_| {
                panic!(
                    "ERROR: Could not parse CUSTOM_EPOCH '{}' as an RFC-3339/ISO-8601 datetime.",
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
            "ERROR: Could not generate a SystemTime custom epoch from milliseconds timestamp '{}'",
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
    let mut vector_ids: Vec<u64> = vec![0; args.number.unwrap()];
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
            .expect("ERROR: Failed to get elapsed time.")
            .as_nanos();
        for (index, element) in vector_ids.into_iter().enumerate() {
            println!("{}: {}", index, element);
        }
        println!(
            "It took {} nanoseconds, time per id: {:.2} ns",
            elapsed,
            elapsed as f64 / args.number.unwrap() as f64
        );
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
