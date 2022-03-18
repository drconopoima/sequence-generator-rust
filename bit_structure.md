# Bit Structure

## Default Parameters:

Retrieving a value:

```sh
sequence-generator-rust --node-id 505
```

Result in decimal:

```text
0: 731536357192630777
```

Result converted into binary (60 signifficant bits):

```text
101000100110111100000111000011100110110000000000000111111001
```

Result in binary zero-padded to 64 bits:

```text
0000101000100110111100000111000011100110110000000000000111111001
```

Right-most 9 bits in binary representing the node-id:

```text
111111001
```

In decimal representation, 505, which we assigned over command line for the host/worker ID.

The following 11 bits, right-to-left, represent the sequence number:

```text
00000000000
```

We are analyzing the element 0 of the generated sequence, which is represented by 0 in both binary/decimal.

The left-most 44 bits represent a custom epoch in tenths of millisecond since 2020-01-01T00:00:00Z

```text
00001010001001101111000001110000111001101100
```

In decimal:

```text
697647435372
```

In seconds:

```text
69764743.5372
```

Converting into date with GNU coreutils `date` utility:

```sh
TZ=UTC date -d "2020-01-02 + 69764743.5372 seconds" --rfc-3339=s
```

Results into:

```text
2022-03-19 11:05:43+00:00
```
