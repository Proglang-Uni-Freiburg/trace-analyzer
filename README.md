# Trace-Analyzer

Tool written in Rust to check traces in STD-format for wellformed-ness.

```shell
cargo run -- --input input/std/Sor.std --normalize
```

| CLI argument          | Required | Info                                      |
|-----------------------|----------|-------------------------------------------|
| `-i` or `--input` \<path>    | True     | Path to the `.std` file                   |
| `-n` or `--normalize` | False    | If the trace needs to be normalized first |
