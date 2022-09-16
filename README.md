# sequence-generator-rust

64-bit IDs sequence generator based on the concepts outlined in Twitter Server's ID (formerly snowflake). Build on Rust

## Table of Contents

- [Installation & build](#installation)
- [Description](#description)
- [Usage](#usage)
- [Library](#Library)
- [Support](#support)
- [Contributing](#contributing)

## Installation

```sh
  git clone https://github.com/drconopoima/sequence-generator-rust.git
  cd sequence-generator-rust
  cargo build --release
```

The binary was generated under `target/release/sequence-generator-rust`

## Description

You can generate sequential IDs based on timestamp, sequence number and node/worker ID (based on Twitter snowflake):

By default this package format defines:

- the right-most 9 bits are used to store worker and/or host information (up to 512)
- subsequently, 11 bits are used to store a sequence number (up to 2048)
- the left-most, 44 bits are used to store a custom epoch with precision of 10 samples every millisecond (10^-1). That's enough to store 55 years from a custom epoch
- There are no bits left unused.
- Custom epoch is set to the beginning of current decade (2020-01-01)

## Usage

Generate a single sequence number as follows, with a worker-id set up from `.env` file (default 0):

```sh
$ cargo run \
  0: 731587959438966784
```

Generate many sequence values (`-n|--number`), provide a custom worker id (`--node-id`), and measure the time taken (`-d|--debug`):

```sh
cargo run --release -- -n 8 --node-id 505 --debug
```

```text
0: 731586108621586937
1: 731586108621587449
2: 731586108621587961
3: 731586108621588473
4: 731586108621588985
5: 731586108621589497
6: 731586108621590009
7: 731586108621590521
It took 661 nanoseconds
```

Each one of the parameters for the sequence are customizable.

By default the original Twitter snowflake format defines:

- 1 bit left unused (sign)
- 41 bits are used to store a custom epoch with millisecond precision (10^3 microseconds for 69 years from a custom epoch)
- 10 bits are used to store worker and datacenter information (up to 1024)
- 12 bits are used to store a sequence number (up to 4096)
- Uses a custom epoch of 1288834974657 or Nov 04 2010 01:42:54.

You can perfectly and easily recreate Twitter's snowflakes by passing the following command arguments.

```sh
$ cargo run --release -- -n 8 -d --unused-bits 1 --node-id-bits 10 --sequence-bits 12 --micros-ten-power 3 --custom-epoch '2010-11-04T01:42:54Z'  --node-id 128
0: 137870923482005632
1: 137870923482006656
2: 137870923482007680
3: 137870923482008704
4: 137870923482009728
5: 137870923482010752
6: 137870923482011776
7: 137870923482012800
It took 571 nanoseconds
```

The specific structure of the integers at the binary level includes:

- The left-most bits (customizable, by default none) might be unused and set to 0.
- The second group of bits store the timestamp in a custom exponential by microseconds (by default `44 bits` and sampling every `100 mcs`, equivalent to argument `--micros-ten-power 2`). You cannot customize number of bits of the timestamp directly, but by indirectly setting different values for other bit groups.
- The third group of bits store the sequence (by default `11 bits`)
- The right-most group of bits store the host/worker ID (by default `9 bits`)

You can also customize by `dotenv` file. Copy the file `.env-example` into `.env`

```sh
cp .env-example .env
```

And change the example values to your liking.

The precedence of parameters assigned through the command-line launch arguments is the highest, whichever are not assigned can be retrieved by use of a `.env` file, and if still unassigned parameters remains, then default values described above are used.

The only supported custom epoch format is `RFC-3339/ISO-8601` both as CLI argument and from the dotenv file.

Check a detailed analysis for a generated value in the [auxiliar bit structure analysis](bit_structure.md)

## Benchmarking

See [auxiliar benchmarking notes](benchmarking.md)
## Library

```rust
use std::time::UNIX_EPOCH;
use ::sequence_generator::*;

let custom_epoch = UNIX_EPOCH;  // SystemTime object representing custom epoch time. Use checked_add(Duration) for different time
let node_id_bits = 10;          // 10-bit node/worker ID
let sequence_bits = 12;         // 12-bit sequence
let unused_bits = 1;            // unused (sign) bits at the start of the ID. 1 or 0 generally
let micros_ten_power = 3;       // Operate in milliseconds (10^3 microseconds)
let node_id = 500;              // Current worker/node ID
let cooldown_ns = 1500;         // initial time in nanoseconds for exponential backoff wait after sequence is exhausted

// Generate SequenceProperties
let properties = sequence_generator::SequenceProperties::new(
        custom_epoch,
        node_id_bits,
        node_id,
        sequence_bits,
        micros_ten_power,
        unused_bits,
        cooldown_ns,
    );

// Generate an ID
let id = sequence_generator::generate_id(&properties).unwrap();
// Decode ID
// Timestamp
let timestamp_micros = sequence_generator::decode_id_unix_epoch_micros(id, &properties);
// Sequence
let sequence = sequence_generator::decode_sequence_id(id, &properties);
// Node ID
let id_node = sequence_generator::decode_node_id(id, &properties);
```

## Support

Please [open an issue](https://github.com/drconopoima/sequence-generator-rust/issues/new) for support.

## Changelog

See [changelog](CHANGELOG.md)

## Contributing

Please contribute using [Github Flow](https://guides.github.com/introduction/flow/). Create a branch, add commits, and [open a pull request](https://github.com/drconopoima/sequence-generator-rust/compare/).
