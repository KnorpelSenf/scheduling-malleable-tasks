use crate::algo::Job;
use rand::Rng;

pub fn jobs(n: i32, min_p: usize, max_p: usize) -> Vec<Job> {
    (0..n)
        .map(|index| Job {
            id: index,
            index: index.try_into().unwrap(),
            processing_times: (0..n)
                .map(|_| {
                    let p = rand::rng().random_range(min_p..max_p);
                    p.try_into().unwrap()
                })
                .collect(),
        })
        .collect()
}
