use crate::algo::{Constraint, Instance, Job};
use csv::{ReaderBuilder, Writer};

pub fn read(job_file: &str, constraint_file: &str) -> Instance {
    let mut rdr = ReaderBuilder::new()
        .from_path(job_file)
        .expect("could not read job CSV");
    let headers = rdr.headers().expect("no headers in job file");
    let header_count = headers.len();
    if header_count <= 1 {
        panic!("too few columns!");
    }
    if headers.iter().next().is_none_or(|name| name != "id") {
        panic!("first column is not id");
    }
    let processor_count = header_count - 1;
    let jobs = (1..)
        .zip(rdr.records())
        .map(|(index, record)| {
            let record = record.unwrap_or_else(|e| panic!("cannot parse record {index}: {:#?}", e));
            let id: i32 = record
                .get(0)
                .unwrap_or_else(|| panic!("missing id in row {index}"))
                .parse()
                .unwrap_or_else(|e| panic!("bad id in row {index}: {:#?}", e));
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
                                panic!("bad processing time in cell at {index}:{column}: {:#?}", e)
                            })
                        })
                        .collect(),
                },
            )
        })
        .collect::<Vec<_>>();
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
    let constraints = (1..)
        .zip(rdr.records())
        .map(|(index, record)| {
            let record = record.unwrap_or_else(|e| panic!("cannot parse record {index}: {:#?}", e));
            let left: i32 = record
                .get(0)
                .unwrap_or_else(|| panic!("missing left side of constraint in row {index}"))
                .parse()
                .unwrap_or_else(|e| {
                    panic!("bad id in left side of constraint in row {index}: {:#?}", e)
                });
            let right: i32 = record
                .get(1)
                .unwrap_or_else(|| panic!("missing right side of constraint in row {index}"))
                .parse()
                .unwrap_or_else(|e| {
                    panic!(
                        "bad id in right side of constraint in row {index}: {:#?}",
                        e
                    )
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

pub fn write(job_file: &str, constraint_file: &str, instance: Instance) {
    let mut wtr = Writer::from_path(job_file).expect("could not write job CSV");
    let headers = std::iter::once("id".to_string())
        .chain((0..instance.processor_count).map(|i| format!("p{}", i)));
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
    wtr.write_record(&["id0", "id1"])
        .expect("could not write headers");
    for Constraint(l, r) in instance.constraints {
        wtr.write_record(std::iter::once(l.to_string()).chain(std::iter::once(r.to_string())))
            .expect("could not write constraint");
    }
    wtr.flush().expect("could not flush constraint CSV");
}
