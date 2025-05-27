# Scheduling Malleable Tasks

A pure Rust implementation of *Scheduling and Packing Malleable and Parallel Tasks With Precedence Constraints of Bounded Width* (2009) by Günther et al.

TODO: elaborate

## Setting Log Levels

By default, the CLI only outputs a few progress bars.

If you want to debug the program, you can tell the CLI to output more detailed logs via the `RUST_LOG` environment variable.
The possible values are listed below.

```sh
export RUST_LOG=error # (currently unused)
export RUST_LOG=warn # (currently unused)
export RUST_LOG=info # prints basic info about the problem instance and measures computation time
export RUST_LOG=debug # provides detailed logs about individual processing steps, disables progress bar
export RUST_LOG=trace # cranks out ~1G of logging data when solving instances with 5000+ jobs
```

## Further Information

The main entrypoint of the CLI is in `src/main.rs`.

The implementation of the paper happens entirely in `src/algo.rs`.
Check it out.
