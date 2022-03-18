# Benchmarking

## Methodology: Top 2% lows

For reproducibility, testing is repeated 100 times and only recorded top 2% lows, which is then repeated 10 times.

The highest CPU scheduler priority of -19 is set with the nice command.

```sh
for j in $(seq 1 10); do
  for i in $(seq 1 100); do
    sudo nice -n -19 sudo -u ljdm ./target/release/sequence-generator-rust -n 163840 -d --node-id 128 2>/dev/null | grep nanoseconds;
   done | awk '{ print $3 }' | sort -n | head -n 2;
done;
```

## Results

### On laptop hardware (Ryzen 5 3550H on Ubuntu 20.04)

Top 2% lows with default parameters average of 59.88 nanoseconds.

When using Twitter snowflake's default sequence bits, top 2% lows average 62.01 nanoseconds.

I have not extensively tested other sets of parameters.

Different hardware would perform differently, and individual optimization is recommended.
