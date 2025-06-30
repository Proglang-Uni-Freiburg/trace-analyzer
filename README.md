# Trace-Analyzer

Tool written in Rust to check traces in STD-format for well-formedness.

```shell
# normalize input and just check of well-formedness violations
cargo run -- --input input/Bensalem.data --normalize

# normalize input and check for violations and generate a graphical representation in GraphViz syntax
cargo run -- --input input/Bensalem.data --normalize --graph

# normalize input and check for violations and analyze via lock dependencies
cargo run -- --input input/Bensalem.data --normalize --lock-dependencies
```

| CLI argument                  | Required | Info                                                                                                       |
|-------------------------------|----------|------------------------------------------------------------------------------------------------------------|
| `-i` or `--input` \<path>     | True     | Path to the `.std` file                                                                                    |
| `-n` or `--normalize`         | False    | If the trace needs to be normalized first                                                                  |
| `-g` or `--graph`             | False    | If a graphical representation of the trace should be generated (HIGH memory usage, beware at large traces) |
| `-l` or `--lock-dependencies` | False    | If a trace should be checked via lock dependencies (HIGH memory usage, beware at large traces)             |

>
