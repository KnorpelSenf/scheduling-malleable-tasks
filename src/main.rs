use std::{fs, io::Write, path, time::Instant};

use algo::{Instance, Schedule, ScheduledJob};
use render::render_schedule;

use clap::{Parser, Subcommand};
use open::that as open_that;

mod algo;
mod dp;
mod files;
mod generate;
mod ilp;
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
    SolveDp {
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
    /// Solves a given instance of the scheduling problem
    SolveIlp {
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
        #[arg(short)]
        n: usize,

        /// Number of processors
        #[arg(short)]
        m: usize,

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

        /// Monotonically increasing processing times
        #[arg(long)]
        concave: bool,
    },
}

fn main() {
    match &Cli::parse().command {
        &Commands::SolveDp {
            ref job_file,
            ref constraint_file,
            svg,
            open,
        } => {
            let schedule = run_algo(dp::schedule, job_file, constraint_file);
            process_schedule(schedule, job_file, constraint_file, svg, open);
        }
        &Commands::SolveIlp {
            ref job_file,
            ref constraint_file,
            svg,
            open,
        } => {
            let schedule = run_algo(ilp::schedule, job_file, constraint_file);
            process_schedule(schedule, job_file, constraint_file, svg, open);
        }
        &Commands::Generate {
            n,
            m,
            min: min_p,
            max: max_p,
            omega,
            min_chain,
            max_chain,
            ref job_file,
            ref constraint_file,
            concave,
        } => {
            assert!(n >= 1, "n must be at least 1");
            assert!(min_p >= 1, "min_p must be at least 1");
            assert!(max_p >= min_p, "max_p must be at least min_p");
            assert!(omega >= 1, "omega must be at least 1");
            assert!(omega <= n as usize, "omega must be at most n");
            assert!(min_chain >= 1, "min_chain must be at least 1");
            assert!(
                max_chain >= min_chain,
                "max_chain must be at least min_chain"
            );
            assert!(max_chain <= n as usize, "max_chain must be at most n");
            assert!(
                min_chain * omega <= n as usize,
                "min_chain * omega must be at at most n"
            );
            assert!(
                max_chain * omega >= n as usize,
                "max_chain * omega must be at at least n"
            );

            let instance =
                generate::instance(n, m, min_p, max_p, omega, min_chain, max_chain, concave);
            files::write(job_file, constraint_file, instance);
        }
    }
}

fn run_algo<T: FnOnce(Instance) -> Schedule>(
    algo: T,
    job_file: &str,
    constraint_file: &str,
) -> Schedule {
    let instance = files::read(job_file, constraint_file);

    let before = Instant::now();
    let schedule = algo(instance);
    let duration = before.elapsed();
    println!(
        "Needed {:?} to schedule {} jobs on {} processors for {} seconds",
        duration,
        schedule.jobs.len(),
        schedule.processor_count,
        schedule
            .jobs
            .iter()
            .map(|job| job.start_time + job.processing_time())
            .max()
            .unwrap_or(0)
    );
    schedule
}

fn process_schedule(
    schedule: Schedule,
    job_file: &str,
    constraint_file: &str,
    svg: bool,
    open: bool,
) {
    if svg {
        let rendered = render_schedule(schedule);

        fs::create_dir_all("./schedules/").expect("cannot create directory ./schedules");
        let path = generate_filename(job_file, constraint_file);
        let mut file = fs::File::create(path.clone())
            .unwrap_or_else(|e| panic!("cannot create file {path}: {e}"));
        file.write_all(rendered.as_bytes())
            .unwrap_or_else(|e| panic!("cannot write to file {path}: {e}"));
        println!("Result is written to {path}");

        if open {
            println!("Opening file ...");
            if let Err(e) = open_that(&path) {
                eprintln!("Could not open file {path}: {:#?}", e);
            }
        }
    } else {
        println!();
        if open {
            println!("  hint: Ignored --open because no schedule file was written");
        }
        println!("  hint: Specify --svg to write a schedule file");
    }
}

fn generate_filename(job_file: &str, constraint_file: &str) -> String {
    let job_file = path::Path::new(job_file)
        .file_stem()
        .unwrap_or_else(|| panic!("Cound not get filename of {job_file}"))
        .to_str()
        .expect("invalid UTF-8 in job file name");
    let constraint_file = path::Path::new(constraint_file)
        .file_stem()
        .unwrap_or_else(|| panic!("Cound not get filename of {constraint_file}"))
        .to_str()
        .expect("invalid UTF-8 in job file name");
    format!("./schedules/{job_file}_{constraint_file}_schedule.svg")
}
