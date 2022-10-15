# Benchmarking

## Methodology: Top 2% lows

For reproducibility, testing is repeated 100 times and only recorded top 2% lows, which is then repeated 10 times.

The highest CPU scheduler priority of -19 is set with the nice command.

```sh
for j in $(seq 1 10); do
  for i in $(seq 1 100); do
    sudo nice -n -19 sudo -u ljdm ./target/release/sequence_generator -n 163840 -d --node-id 128 2>/dev/null | grep nanoseconds;
   done | awk '{ print $3 }' | sort -n | head -n 2;
done;
```

## Results

### On laptop hardware (Ryzen 5 3550H on Ubuntu 20.04)

Top 2% lows with default parameters average of 62.39 nanoseconds per generated ID.

When using Twitter snowflake's default bit configuration, top 2% lows average 240.10 nanoseconds per ID.

```sh
for j in $(seq 1 10); do
  for i in $(seq 1 100); do
    sudo nice -n -19 ./target/release/sequence_generator -n 163840 -d --unused-bits 1 --node-id-bits 10 --sequence-bits 12 --micros-ten-power 3 --custom-epoch '2010-11-04T01:42:54Z' --node-id 128 2>/dev/null | grep nanoseconds;
  done | awk '{ print $3 }' | sort -n | head -n 2;
done;
```

Twitter Snowflake's values consistently reach the maximum sequence ID and stall for 75% of the time while waiting for the next millisecond.

Different hardware would perform differently, and individual optimization of bit configuration is recommended.
