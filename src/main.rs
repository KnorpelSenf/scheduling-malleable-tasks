use algo::{Constraint, Instance, Job, Schedule, ScheduledJob};
use render::render_schedule;

use clap::{Parser, Subcommand};
use csv::ReaderBuilder;

mod algo;
mod generate;
mod render;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Solves a given instance of the scheduling problem
    Solve {
        /// Solution accuracy
        #[arg(short, long)]
        epsilon: f64,

        /// Number of processors
        #[arg(short, long)]
        m: usize,

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
        n: usize,

        /// Maximum processing time for each job
        #[arg(long)]
        min: usize,

        /// Maximum processing time for each job
        #[arg(long)]
        max: usize,

        /// Output CSV file containing the jobs
        #[arg(short, long)]
        job_file: String,

        /// Constraint width
        #[arg(short, long)]
        omega: usize,

        /// Minimum chain length
        #[arg(long)]
        min_chain: usize,

        /// Maximum chain length
        #[arg(long)]
        max_chain: usize,

        /// Output CSV file containing constraints between jobs
        #[arg(short, long)]
        constraint_file: String,
    },
}

fn main() {
    // println!(
    //     "{}",
    //     render_schedule(&Schedule {
    //         processor_count: 3,
    //         jobs: vec![
    //             (10, vec![5, 3, 2], 2, 0),
    //             (11, vec![6, 4, 3], 3, 6),
    //             (12, vec![3, 1, 1], 1, 3),
    //             (13, vec![7, 2, 1], 1, 1)
    //         ]
    //         .into_iter()
    //         .enumerate()
    //         .map(|(index, (id, processing_times, allotment, start_time))| {
    //             ScheduledJob {
    //                 job: Job {
    //                     id,
    //                     index,
    //                     processing_times,
    //                 },
    //                 allotment: allotment,
    //                 start_time,
    //             }
    //         })
    //         .collect()
    //     })
    // );

    let cli = Cli::parse();

    match &cli.command {
        Commands::Solve {
            job_file,
            constraint_file,
            epsilon,
            m,
            svg,
            open,
        } => {
            let instance = parse_input(job_file, constraint_file);

            let schedule = algo::schedule(instance);
            println!("{}", render_schedule(schedule));
        }
        Commands::Generate {
            n,
            min: min_p,
            max: max_p,
            job_file,
            omega,
            min_chain,
            max_chain,
            constraint_file,
        } => {
            assert!(n < &1, "n must be at least 1");
            assert!(min_p < &1, "min_p must be at least 1");
            assert!(max_p < min_p, "max_p must be at least min_p");
            assert!(omega < &1, "omega must be at least 1");
            assert!(*omega > *n as usize, "omega must be at most n");
            assert!(min_chain < &1, "min_chain must be at least 1");
            assert!(
                max_chain < min_chain,
                "max_chain must be at least min_chain"
            );
            assert!(*max_chain > *n as usize, "max_chain must be at most n");
            assert!(
                *min_chain * omega > *n as usize,
                "min_chain * omega must be at at most n"
            );
            assert!(
                *max_chain * omega < *n as usize,
                "max_chain * omega must be at at least n"
            );

            let jobs = generate::jobs(*n, *min_p, *max_p);
            let chains = generate::chains(*n, *omega, *min_chain, *max_chain);
            todo!("Not implemented yet");
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
