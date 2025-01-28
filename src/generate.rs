use crate::algo::Job;
use rand::seq::SliceRandom;
use rand::Rng;

pub fn jobs(n: usize, min_p: usize, max_p: usize) -> Vec<Job> {
    (0..n)
        .map(|index| Job {
            id: index as i32,
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

pub fn chains(n: usize, omega: usize, min_chain: usize, max_chain: usize) -> Vec<(usize, usize)> {
    let mut indices = (1..n).collect::<Vec<_>>();
    indices.shuffle(&mut rand::rng());

    let mut cuts = indices[0..omega].to_vec();
    cuts.sort();
    cuts.ensure_slice_size(min_chain, max_chain);

    let mut jobs = (0..n).collect::<Vec<_>>();
    jobs.shuffle(&mut rand::rng());

    jobs.windows(2)
        .map(|jobs| (jobs[0], jobs[1]))
        .filter(|(j1, _)| cuts.contains(&j1))
        .collect()
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
