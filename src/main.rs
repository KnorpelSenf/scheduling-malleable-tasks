use algo::{Constraint, Instance, Job, Schedule, ScheduledJob};
use render::render_schedule;

use clap::{Parser, Subcommand};
use csv::ReaderBuilder;

mod algo;
mod render;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Solves a given instance of the scheduling problem
    Solve {
        /// Solution accuracy
        #[arg(short, long)]
        epsilon: f64,

        /// Input CSV file containing jobs in the format "id,p_1,...,p_m" where each
        /// column p_i contains the processing time if the job were to be executed
        /// on i machines.
        #[arg(short, long)]
        job_file: String,

        /// Input CSV file containing constraints between jobs in the format
        /// "id0,id1" where each line expresses that the job with id0 is less than
        /// the job with id1.
        #[arg(short, long)]
        constraint_file: String,

        /// Render the schedule to an SVG file in the directory "schedules"
        #[arg(long)]
        svg: bool,

        /// Open the rendered SVG if created
        #[arg(long)]
        open: bool,
    },
    /// Generates a random instance of the scheduling problem
    Generate {
        /// Number of jobs to generate
        #[arg(short, long)]
        jobs: usize,

        /// Number of processors to generate
        #[arg(short, long)]
        processors: usize,

        /// Maximum processing time for each job
        #[arg(short, long)]
        max_time: usize,

        /// Output CSV file to write the generated instance to
        #[arg(short, long)]
        output: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Solve {
            job_file,
            constraint_file,
            epsilon,
            svg,
            open,
        }) => {
            let instance = parse_input(job_file, constraint_file);

            let schedule = algo::schedule(instance);
            render_schedule(&schedule);
        }
        Some(Commands::Generate {
            jobs,
            processors,
            max_time,
            output,
        }) => {
            //let instance = algo::generate(*jobs, *processors, *max_time);
            todo!("Not implemented yet");
        }
        None => {
            println!("Please use the 'solve' or 'generate' subcommands");
        }
    }
}

fn parse_input(job_file_path: &str, constraint_file_path: &str) -> Instance {
    let mut rdr = ReaderBuilder::new()
        .from_path(job_file_path)
        .expect("could not read job CSV");
    let headers = rdr.headers().expect("no headers in job file");
    let header_count = headers.len();
    if header_count <= 1 {
        panic!("too few columns!");
    }
    if headers.iter().next().unwrap_or_default() != "id" {
        panic!("first column is not id");
    }
    let processor_count = header_count - 1;
    let jobs = (1..)
        .zip(rdr.records())
        .map(|(index, record)| {
            let record = record.unwrap_or_else(|e| panic!("cannot parse record {index}: {:#?}", e));
            Job {
                id: record
                    .get(0)
                    .unwrap_or_else(|| panic!("missing id in row {index}"))
                    .parse()
                    .unwrap_or_else(|e| panic!("bad id in row {index}: {:#?}", e)),
                index,
                processing_times: record
                    .iter()
                    .enumerate()
                    .skip(1)
                    .map(|(column, cell)| {
                        cell.parse().unwrap_or_else(|e| {
                            panic!("bad processing time in cell at {index}:{column}: {:#?}", e)
                        })
                    })
                    .collect(),
            }
        })
        .collect();
    let mut rdr = ReaderBuilder::new()
        .from_path(constraint_file_path)
        .expect("cound not read constraints CSV");
    assert_eq!(
        rdr.headers()
            .expect("no headers in constraint file")
            .iter()
            .collect::<Vec<&str>>(),
        vec!["id0", "id1"]
    );
    let constraints = (1..)
        .zip(rdr.records())
        .map(|(index, record)| {
            let record = record.unwrap_or_else(|e| panic!("cannot parse record {index}: {:#?}", e));
            Constraint(
                record
                    .get(0)
                    .unwrap_or_else(|| panic!("missing left side of constraint in row {index}"))
                    .parse()
                    .unwrap_or_else(|e| {
                        panic!("bad id in left side of constraint in row {index}: {:#?}", e)
                    }),
                record
                    .get(1)
                    .unwrap_or_else(|| panic!("missing right side of constraint in row {index}"))
                    .parse()
                    .unwrap_or_else(|e| {
                        panic!(
                            "bad id in right side of constraint in row {index}: {:#?}",
                            e
                        )
                    }),
            )
        })
        .collect();

    Instance {
        processor_count,
        jobs,
        constraints,
    }
}
