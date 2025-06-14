// Small helper functions to generate random instances of the scheduling problem.

use crate::algo::{Constraint, Instance, Job};
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::Rng;
use std::cmp;

#[expect(clippy::too_many_arguments)]
/// Generates a randomized instance of the scheduling problem with `n` jobs and `m` processors.
/// The processing times of the jobs are randomly generated within the range of `min_p` to `max_p`.
/// `omega` many chains are created, with each chain having a length between `min_chain` and `max_chain`.
/// If `concave` is true, the processing times follow the concave 1/n function, otherwise they are randomly distributed.
pub fn instance(
    n: usize,
    m: usize,
    min_p: i32,
    max_p: i32,
    omega: usize,
    min_chain: usize,
    max_chain: usize,
    concave: bool,
) -> Instance {
    Instance {
        processor_count: m,
        jobs: if concave {
            jobs_concave(n, m as i32, min_p, max_p)
        } else {
            jobs(n, m, min_p, max_p)
        },
        constraints: constraints(n, omega, min_chain, max_chain),
        max_time: n as i32 * max_p,
    }
}

/// Generates jobs following a concave 1/n processing time function.
fn jobs_concave(n: usize, m: i32, min_p: i32, max_p: i32) -> Vec<Job> {
    (0..n)
        .map(|index| {
            let p = rand::rng().random_range(min_p..max_p);
            let cutoff = rand::rng().random_range(1..=m);
            Job {
                index,
                processing_times: (1..=m).map(|i| p / cmp::min(i, cutoff)).collect(),
            }
        })
        .collect()
}

/// Generates jobs with random processing times for each allotment.
fn jobs(n: usize, m: usize, min_p: i32, max_p: i32) -> Vec<Job> {
    (0..n)
        .map(|index| Job {
            index,
            processing_times: (1..=m)
                .map(|_| rand::rng().random_range(min_p..max_p))
                .collect(),
        })
        .collect()
}

/// Generates constraints for the scheduling problem.
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
                .collect()
        })
}

trait SlicesWithSize {
    type T;
    fn ensure_slice_size(&mut self, min: Self::T, max: Self::T);
}

/// Small helper implementation to ensure that all elements are at least min and at most
/// max apart from each other.
impl<E> SlicesWithSize for Vec<E>
where
    E: Copy
        + Ord
        + std::ops::Add<Output = E>
        + std::ops::Sub<Output = E>
        + std::ops::Mul<usize, Output = E>,
{
    type T = E;
    /// iterate over self and check if two consecutive value are within min and max apart
    /// if not, increase or decrease the second value to fit the bounds
    fn ensure_slice_size(&mut self, min: E, max: E) {
        for i in 0..self.len() - 1 {
            let max_remaining = cmp::max(max, min * (self.len() - i));
            let diff = self[i + 1] - self[i];
            if diff < min {
                self[i + 1] = self[i] + min;
            } else if diff > max_remaining {
                self[i + 1] = self[i] + max_remaining;
            }
        }

        // just make sure we did not mess up
        let last = self.len() - 1;
        let diff = self[last] - self[last - 1];
        assert!(
            min <= diff && diff <= max,
            "unfortunate random values, cannot handle"
        );
    }
}
