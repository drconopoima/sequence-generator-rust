# sequence-generator-rust

64-bit IDs sequence generator based on the concepts outlined in Twitter Server's ID (formerly snowflake). Build on Rust

## Table of Contents

- [Installation & build](#installation)
- [Usage](#usage)
- [Support](#support)
- [Contributing](#contributing)

## Installation

```sh
  git clone https://github.com/drconopoima/sequence-generator-rust.git
  cd sequence-generator-rust
  cargo build --release
```

The binary was generated under `target/release/sequence-generator-rust`

## Usage

You can generate sequential IDs based on timestamp, sequence number and node/worker ID (based on Twitter snowflake):

By default this package format defines:

- 44 bits are used to store a custom epoch with precision of 10 samples every millisecond (10^-1). That's enough to store 55 years from a custom epoch
- 9 bits are used to store worker and/or host information (up to 512)
- 11 bits are used to store a sequence number (up to 2048)
- There are no bits left unused.
- Custom epoch is set to the beginning of current decade (2020-01-01)

Expected output would be like the following

```sh
$ cargo run \
  0: 344656800457424896
```

You can just as easily generate more than one ID from the CLI (`-n|--number`), as well as measure the time taken by the batch `-d|--debug`, providing a worker id of 0 (`--node-id`).

```sh
$ cargo run --release -- -n 8 --node-id 505 --debug \
  0: 344680846955905529
  1: 344680846955906041
  2: 344680846955906553
  3: 344680846955907065
  4: 344680846955907577
  5: 344680846955908089
  6: 344680846955908601
  7: 344680846955909113
  It took 611 nanoseconds
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

I wouldn't use Twitter's values nowadays. Hardware has advanced enough that with a single thread of a laptop average processor I'm generating each ID in ~60-70 nanoseconds, even when reaching close to the maximum sequence and stalling a bit waiting for the next millisecond, while the Twitter's defaults always generate the full sequence and then stalls for 55% of the time waiting (averaging 135-145ns per ID). Twitter's snowflakes are optimal if the hardware takes 245ns per ID or lower. Furthermore, having tenths of a millisecond precision means more accurate time & fast sorting if coupled with a Radix sort algorithm.

The specific structure of the integers at the binary level includes:

- The first few bits (customizable, by default none) might be unused and set to 0.
- The second group of bits store the timestamp in a custom exponential by microseconds (by default `44 bits` and sampling every `100 mcs`, equivalent to argument `--micros-ten-power 2`). You cannot customize number of bits of the timestamp directly, but by indirectly setting different values for other bit groups.
- The third group of bits store the sequence (by default `11 bits`)
- The last group of bits store the host/worker ID (by default `9 bits`)

You can also customize by `dotenv` file. Copy the file `.env-example` into `.env`

```sh
cp .env-example .env
```

And change the example values to your liking.

The prevalence of the dotenv values is higher than CLI parameters passed.

The only supported custom epoch format is `RFC-3339/ISO-8601` both as CLI argument and from the dotenv file.

## Support

Please [open an issue](https://github.com/drconopoima/sequence-generator-rust/issues/new) for support.

## Contributing

Please contribute using [Github Flow](https://guides.github.com/introduction/flow/). Create a branch, add commits, and [open a pull request](https://github.com/drconopoima/sequence-generator-rust/compare/).
