use algo::{Instance, Job, Schedule};
use clap::Parser;
use csv::ReaderBuilder;

mod algo;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
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
}

fn main() {
    let args = Args::parse();
    let instance = parse_input(&args.job_file, &args.constraint_file);

    let schedule = algo::schedule(instance);
    print_schedule(&schedule);
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
    let jobs = rdr
        .records()
        .enumerate()
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

    Instance {
        processor_count,
        jobs,
        constraints: vec![], // TODO: parse file
    }
}

fn print_schedule(_schedule: &Schedule) {}
