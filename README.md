# Scheduling Malleable Tasks

A Rust implementation of three different algorithms for scheduling malleable tasks with precendence constraints.

In this project, we abbreviate the three papers with DP, LP, and ILP.

1. LP: [An Approximation Algorithm for Scheduling Malleable Tasks Under General Precedence Constraints](https://doi.org/10.1007/11602613_25) by Jansen et al.
2. ILP: [Scheduling Malleable Tasks With Precedence Constraints](https://doi.org/10.1016/j.jcss.2011.04.003) by Jansen et al.
3. DP: [Scheduling and Packing Malleable and Parallel Tasks With Precedence Constraints of Bounded Width](https://doi.org/10.1007/s10878-012-9498-3) by Günther et al.

This repository includes:

- a problem instance generation script
- solvers for all three above algorithms
- a somewhat broken SVG renderer for solutions
- several sample problems instances
- an evaluation script to generate and solve many problems in a grid search
- a postprocessing script for ease of visualisation
- a devcontainer setup

## System Setup

## Easy Way (Devcontainers)

The easiest way to set up your system is to [install Docker](https://docs.docker.com/engine/install/) on your Linux system.
You can then open this project inside the provided [devcontainer](https://containers.dev/) and you are ready to go.

## Hard Way (Installing System Dependencies Manually)

If you do not wish to use Docker for some reason, you will need to replicate the same system setup on your host machine.
If you run Debian Bookworm, you can do this by following the below steps.

1. Install [the Rust toolchain](https://www.rust-lang.org/tools/install)
2. Install [Deno](https://docs.deno.com/runtime/getting_started/installation/)
3. Install the packages `coinor-libcbc-dev` and `clang` using `apt-get`.

## Building

This is a Rust project (with some TypeScript scripts around it).

The TypeScript script do not require a build step at all.

The Rust parts can be compiled by running

```sh
cargo build
```

in the top-level of this repository.

## Generating Instances

You can generate instances using `cargo run -- generate`.
The following options are available.

- number of machines
- number of jobs
- minimum and maximum job duration
- number of chains
- minimum and maximum chain length
- enable concave processing time functions

Run `cargo run -- generate --help` to see all options.

```sh
$ cargo run -q -- generate -h
Generates a random instance of the scheduling problem

Usage: scheduling-malleable-tasks generate [OPTIONS] -n <N> -m <M> --min <MIN> --max <MAX> --job-file <JOB_FILE> --omega <OMEGA> --min-chain <MIN_CHAIN> --max-chain <MAX_CHAIN> --constraint-file <CONSTRAINT_FILE>

Options:
  -n <N>
          Number of jobs to generate
  -m <M>
          Number of processors
      --min <MIN>
          Maximum processing time for each job
      --max <MAX>
          Maximum processing time for each job
  -j, --job-file <JOB_FILE>
          Output CSV file containing the jobs
  -o, --omega <OMEGA>
          Constraint width
      --min-chain <MIN_CHAIN>
          Minimum chain length
      --max-chain <MAX_CHAIN>
          Maximum chain length
  -c, --constraint-file <CONSTRAINT_FILE>
          Output CSV file containing constraints between jobs
      --concave
          Monotonically decreasing processing times using the concave function 1 / l
  -h, --help
          Print help
  -V, --version
          Print version
```

## Running the Solver

The CLI contains the implementations of three different scheduling algorithms.
All of them can be run using `cargo run -- <algorithm> <arguments>`.

The allowed values for `<algorithm>` are `solve-dp`, `solve-lp`, and `solve-ilp`.

All algorithms require you to specifiy a job file and a constraint file.
They also support SVG generation and can optionally open the generated SVG automatically.

### Scheduling via DP

The dynamic program is the fastest and most scalable algorithm, but it also delivers schedules with the longest makespan.

You can run it using `cargo run -- solve-dp` with the following options.

```sh
$ cargo run -q -- solve-dp -h
Solves a given instance of the scheduling problem using a dynamic program

Usage: scheduling-malleable-tasks solve-dp [OPTIONS] --job-file <JOB_FILE> --constraint-file <CONSTRAINT_FILE>

Options:
  -j, --job-file <JOB_FILE>
          Input CSV file containing jobs in the format `id,p_1,...,p_m` where each column `p_i` contains the processing time if the job were to be executed on i machines
  -c, --constraint-file <CONSTRAINT_FILE>
          Input CSV file containing constraints between jobs in the format "id0,id1" where each line expresses that the job with id0 is less than the job with id1
      --svg
          Render the schedule to an SVG file in the directory "schedules"
      --open
          Open the rendered SVG if created
  -h, --help
          Print help
  -V, --version
          Print version
```

### Scheduling via LP

The linear program is the slowest and oldest of the three algoritms.

You can run it using `cargo run -- solve-lp` with the following options.

```sh
$ cargo run -q -- solve-lp -h
Solves a given instance of the scheduling problem using a linear program

Usage: scheduling-malleable-tasks solve-lp [OPTIONS] --job-file <JOB_FILE> --constraint-file <CONSTRAINT_FILE>

Options:
  -j, --job-file <JOB_FILE>
          Input CSV file containing jobs in the format `id,p_1,...,p_m` where each column `p_i` contains the processing time if the job were to be executed on i machines
  -c, --constraint-file <CONSTRAINT_FILE>
          Input CSV file containing constraints between jobs in the format "id0,id1" where each line expresses that the job with id0 is less than the job with id1
      --svg
          Render the schedule to an SVG file in the directory "schedules"
      --open
          Open the rendered SVG if created
      --compress
          Remove idle times from schedule in a postprocessing step
  -h, --help
          Print help
  -V, --version
          Print version
```

Note that the algoritm generates empty time slices with no jobs scheduled.
In an optional postprocessing step, our implementation can remove the idle times and compress the schedule.

This is not part of the paper.
In order to stay as close as possible to the original piece of research, this flag was not set in the evaluation.

### Scheduling via ILP

The integer linear program delivers the best makespan of the three algoritms.
(Remember that the paper describes an ILP but the algorithm actually solves the relaxed LP variant of it, which is described in the same paper.
We do not actually need to solve a linear program with integer constraints.)

You can run it using `cargo run -- solve-ilp` with the following options.

```sh
$ cargo run -q -- solve-ilp -h
Solves a given instance of the scheduling problem using an integer linear program

Usage: scheduling-malleable-tasks solve-ilp [OPTIONS] --job-file <JOB_FILE> --constraint-file <CONSTRAINT_FILE>

Options:
  -j, --job-file <JOB_FILE>
          Input CSV file containing jobs in the format `id,p_1,...,p_m` where each column `p_i` contains the processing time if the job were to be executed on i machines
  -c, --constraint-file <CONSTRAINT_FILE>
          Input CSV file containing constraints between jobs in the format "id0,id1" where each line expresses that the job with id0 is less than the job with id1
      --svg
          Render the schedule to an SVG file in the directory "schedules"
      --open
          Open the rendered SVG if created
      --compress
          Remove idle times from schedule in a postprocessing step
  -h, --help
          Print help
  -V, --version
          Print version
```

Note that similarly to the linear program, the algoritm generates empty time slices with no jobs scheduled.
In an optional postprocessing step, our implementation can remove the idle times and compress the schedule.

This is not part of the paper.
In order to stay as close as possible to the original piece of research, this flag was not set in the evaluation.

## Running the Evaluation

An evaluation script is provided in `instances/eval/eval.sh`.

You can run it without any arguments or build steps in order to measure the performance of the dynamic program.
The script will perform a grid search over many problem instances using 16 cores.

It can be easily adjusted (check the comments) in order to measure all three algorithms over varying problem spaces.

## Running the Postprocessing Script

The CLI itelf outputs four columns of values, but a typical visualisation only needs the the number of jobs, the duration, and the makespan (in that order).
The machine count is often constant and can be dropped.

Run `deno -A instances/eval/reorder.ts <path-to-csv>` to

- read the CSV file
- select the columns `n`, `ms`, and `makespan` in that order
- overwrite the same CSV file

Drop the `-A` flag to inspect and grant script permissions manually.

## Setting Log Levels

By default, the CLI only outputs four values.
Many CLI runs can be concatenated to form a CSV file.

If you want to see some actual output or even debug the program, you can tell the CLI to output more detailed logs via the `RUST_LOG` environment variable.
The possible values are listed below.

```sh
export RUST_LOG=error # logs SVG file write errors
export RUST_LOG=warn # (currently unused)
export RUST_LOG=info # prints basic info about the solving process and the solution
export RUST_LOG=debug # provides more detailed logs about the problem and some intermediate variables
export RUST_LOG=trace # (currently unused)
```

## Source Code Structure

The main entrypoint of the CLI is in `src/main.rs`.
The orchestrations of the algorithms happens there, too.

The problem instance and solution definitions happen in `src/algo.rs`.

The implementations of the papers happen entirely in `src/{dp,lp,ilp}.rs`.
Check them out.

Instance generation is located in in `src/generate.rs` and SVG rendering is in `src/render.rs`.
Finally, file IO happens in `src/files.rs`.
