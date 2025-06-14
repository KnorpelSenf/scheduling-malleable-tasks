// CSV file handling implementations.

use crate::algo::{Constraint, Instance, Job};
use csv::{ReaderBuilder, Writer};

/// Reads a job and constraint CSV file and returns an `Instance`.
pub fn read(job_file: &str, constraint_file: &str) -> Instance {
    let mut rdr = ReaderBuilder::new()
        .from_path(job_file)
        .expect("could not read job CSV");
    let headers = rdr.headers().expect("no headers in job file");
    let header_count = headers.len();
    assert!((header_count > 1), "too few columns!");
    assert!(
        headers.iter().next().is_some_and(|name| name == "id"),
        "first column is not id"
    );
    let processor_count = header_count - 1;
    let jobs = rdr
        .records()
        .enumerate()
        .map(|(index, record)| {
            let row = index + 1;
            let record = record.unwrap_or_else(|e| panic!("cannot parse record {row}: {e:#?}"));
            let id: i32 = record
                .get(0)
                .unwrap_or_else(|| panic!("missing id in row {row}"))
                .parse()
                .unwrap_or_else(|e| panic!("bad id in row {row}: {e:#?}"));
            (
                id,
                Job {
                    index,
                    processing_times: record
                        .iter()
                        .enumerate()
                        .skip(1)
                        .map(|(column, cell)| {
                            cell.parse().unwrap_or_else(|e| {
                                panic!("bad processing time in cell at {row}:{column}: {e:#?}")
                            })
                        })
                        .collect(),
                },
            )
        })
        .collect::<Vec<_>>();

    let n = jobs.len();

    let mut rdr = ReaderBuilder::new()
        .from_path(constraint_file)
        .expect("cound not read constraints CSV");
    assert_eq!(
        rdr.headers()
            .expect("no headers in constraint file")
            .iter()
            .collect::<Vec<&str>>(),
        vec!["id0", "id1"]
    );
    let constraints = rdr
        .records()
        .enumerate()
        .map(|(index, record)| {
            let row = index + 1;
            let record = record.unwrap_or_else(|e| panic!("cannot parse record {row}: {e:#?}"));
            let left: i32 = record
                .get(0)
                .unwrap_or_else(|| panic!("missing left side of constraint in row {row}"))
                .parse()
                .unwrap_or_else(|e| {
                    panic!("bad id in left side of constraint in row {row}: {e:#?}")
                });
            let right: i32 = record
                .get(1)
                .unwrap_or_else(|| panic!("missing right side of constraint in row {row}"))
                .parse()
                .unwrap_or_else(|e| {
                    panic!("bad id in right side of constraint in row {row}: {e:#?}")
                });

            Constraint(
                jobs.iter()
                    .find(|(id, _)| *id == left)
                    .expect("bad left side")
                    .1
                    .index,
                jobs.iter()
                    .find(|(id, _)| *id == right)
                    .expect("bad right side")
                    .1
                    .index,
            )
        })
        .take_while(|Constraint(l, r)| *l < n && *r < n)
        .collect();

    let max_time = jobs.len() as i32
        * jobs
            .iter()
            .map(|job| job.1.processing_times.iter().max().copied().unwrap_or(0))
            .max()
            .unwrap_or(0);

    Instance {
        processor_count,
        jobs: jobs.into_iter().map(|pair| pair.1).collect(),
        constraints,
        max_time,
    }
}

/// Writes an `Instance` to job and constraint CSV files.
pub fn write(job_file: &str, constraint_file: &str, instance: Instance) {
    let mut wtr = Writer::from_path(job_file).expect("could not write job CSV");
    let headers = std::iter::once("id".to_string())
        .chain((0..instance.processor_count).map(|i| format!("p{i}")));
    wtr.write_record(headers).expect("could not write headers");
    for job in instance.jobs {
        wtr.write_record(
            std::iter::once(job.index.to_string())
                .chain(job.processing_times.into_iter().map(|p| p.to_string())),
        )
        .expect("could not write job");
    }
    wtr.flush().expect("could not flush job CSV");

    let mut wtr = Writer::from_path(constraint_file).expect("could not write constraint CSV");
    wtr.write_record(["id0", "id1"])
        .expect("could not write headers");
    for Constraint(l, r) in instance.constraints {
        wtr.write_record(std::iter::once(l.to_string()).chain(std::iter::once(r.to_string())))
            .expect("could not write constraint");
    }
    wtr.flush().expect("could not flush constraint CSV");
}
