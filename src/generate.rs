use crate::algo::{Constraint, Instance, Job};
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::Rng;
use std::cmp;

#[expect(clippy::too_many_arguments)]
pub fn instance(
    n: usize,
    m: usize,
    min_p: usize,
    max_p: usize,
    omega: usize,
    min_chain: usize,
    max_chain: usize,
    concave: bool,
) -> Instance {
    Instance {
        processor_count: m,
        jobs: jobs(n, m, min_p, max_p, concave),
        constraints: constraints(n, omega, min_chain, max_chain),
        max_time: (n * max_p) as i32,
    }
}

fn jobs(n: usize, m: usize, min_p: usize, max_p: usize, concave: bool) -> Vec<Job> {
    if concave {
        return (0..n)
            .map(|index| {
                let p = rand::rng().random_range(min_p..max_p) as f64;
                let cutoff = rand::rng().random_range(0..m);
                Job {
                    index,
                    processing_times: (0..m)
                        .map(|i| (p / cmp::min(i, cutoff) as f64 + 1.0) as i32)
                        .collect(),
                }
            })
            .collect();
    }
    (0..n)
        .map(|index| Job {
            index,
            processing_times: (0..m)
                .map(|_| rand::rng().random_range(min_p..max_p) as i32)
                .collect(),
        })
        .collect()
}

fn constraints(n: usize, omega: usize, min_chain: usize, max_chain: usize) -> Vec<Constraint> {
    let mut indices = (1..n).collect::<Vec<_>>();
    indices.shuffle(&mut rand::rng());

    let mut cuts = indices[0..omega - 1].to_vec();
    cuts.sort_unstable();
    cuts.ensure_slice_size(min_chain, max_chain);

    [0].iter()
        .chain(cuts.iter())
        .chain([n].iter())
        .tuple_windows()
        .fold(vec![], |constraints, (&l, &r)| {
            constraints
                .into_iter()
                .chain(
                    (l..r)
                        .flat_map(|job0| (job0..r).map(move |job1| (job0, job1)))
                        .map(|(left, right)| Constraint(left, right)),
                )
                .collect::<Vec<_>>()
        })
}

trait SlicesWithSize {
    type T;
    fn ensure_slice_size(&mut self, min: Self::T, max: Self::T);
}

impl<E> SlicesWithSize for Vec<E>
where
    E: Copy + PartialOrd + std::ops::Add<Output = E> + std::ops::Sub<Output = E>,
{
    type T = E;
    fn ensure_slice_size(&mut self, min: E, max: E) {
        // iterate over self and check if two consecutive value are within min and max apart
        // if not, increase or decrease the second value to fit the bounds

        for i in 0..self.len() - 1 {
            let diff = self[i + 1] - self[i];
            if diff < min {
                self[i + 1] = self[i] + min;
            } else if diff > max {
                self[i + 1] = self[i] + max;
            }
        }
    }
}
