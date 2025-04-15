use std::{cmp::Ordering, collections::HashSet, hash::Hash};

use crate::algo::{Instance, PartialRelation, Schedule, ScheduledJob};

#[derive(Clone, Debug, PartialEq, Eq)]
/// A state in our dynamic program
struct State {
    /// For each chain, how far have we advanced this chain
    ideal: Vec<usize>,
    /// For each chain, how many machines do we use for the front task, i.e. the job indicated by `ideal`
    allotment: Vec<usize>,
    /// For each chain, how many machines do we use for the front task, i.e. the job indicated by `ideal`
    completion_times: Vec<i32>,
}
impl State {
    fn empty(omega: usize) -> Self {
        Self {
            ideal: vec![0; omega],
            allotment: vec![0; omega],
            completion_times: vec![0; omega],
        }
    }
    // fn start_times(&self, i: usize) -> i32 {
    //     self.ideal[i]
    //         .as_ref()
    //         .map(|ideal| self.completion_times[i] - ideal.processing_time(self.allotment[i]))
    //         .unwrap_or(0)
    // }
    fn add_job(&self, chain: usize, allot: usize, compl: i32) -> Self {
        let mut ideal = self.ideal.clone();
        let mut allotment = self.allotment.clone();
        let mut completion_times = self.completion_times.clone();
        ideal[chain] += 1;
        allotment[chain] = allot;
        completion_times[chain] = compl;
        Self {
            ideal,
            allotment,
            completion_times,
        }
    }
}
impl Hash for State {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ideal.hash(state);
    }
}

#[expect(clippy::needless_pass_by_value)]
pub fn schedule(instance: Instance) -> Schedule {
    let chains = preprocess(&instance);
    let omega = chains.len();
    let initial_state = State::empty(omega);
    let jobs =
        search(&instance, &chains, &initial_state, &mut HashSet::new()).expect("no solution found");
    println!("jobs are {jobs:#?}");
    Schedule {
        processor_count: instance.processor_count,
        jobs,
    }
}

fn search(
    instance: &Instance,
    chains: &Vec<Vec<usize>>,
    state: &State,
    known: &mut HashSet<State>,
) -> Option<Vec<ScheduledJob>> {
    if state.ideal.iter().sum::<usize>() == instance.jobs.len() {
        return Some(vec![]);
    }

    for (chain_index, chain) in chains.iter().enumerate() {
        let ideal = state.ideal[chain_index];
        if ideal == chain.len() {
            continue;
        }
        let new_job_index = chain[ideal];
        let new_job = &instance.jobs[new_job_index];
        for (&processing_time, allotment) in new_job.processing_times.iter().zip(1..) {
            for compl in 0..instance.max_time {
                let new_start_time = compl - processing_time;
                if new_start_time < 0 {
                    continue;
                }

                let mut can_insert = true;
                for (chain_index, &ideal) in
                    state.ideal.iter().filter(|&&ideal| ideal != 0).enumerate()
                {
                    let completion_time = state.completion_times[chain_index];
                    let front_job_index = chains[chain_index][ideal - 1];
                    let front_job = &instance.jobs[front_job_index];

                    // Condition 2
                    if front_job.less_than(&instance.constraints, new_job)
                        && new_start_time < completion_time
                    {
                        can_insert = false;
                        break;
                    }
                    // Condition 3
                    let processing_time = front_job.processing_time(state.allotment[chain_index]);
                    if new_start_time < completion_time - processing_time {
                        can_insert = false;
                        break;
                    }
                }
                if !can_insert {
                    continue;
                }

                // Check if processor count exceeded
                let mut pairs = state
                    .ideal
                    .iter()
                    .filter(|&&ideal| ideal != 0)
                    .enumerate()
                    .flat_map(|(chain_index, &ideal)| {
                        let front_job_index = chains[chain_index][ideal - 1];
                        let front_job = if new_job_index == front_job_index {
                            new_job
                        } else {
                            &instance.jobs[front_job_index]
                        };
                        let completion_time = state.completion_times[chain_index];
                        let start_time = completion_time
                            - front_job.processing_time(state.allotment[chain_index]);
                        let a = allotment as i32;
                        vec![(start_time, a), (completion_time, -a)]
                    })
                    .collect::<Vec<_>>();
                pairs.sort_by_key(|p| p.0);
                let limit = instance.processor_count as i32;
                let mut utilisation = 0;
                for (_, diff) in pairs {
                    utilisation += diff;
                    if utilisation > limit {
                        can_insert = false;
                        break;
                    }
                }
                if !can_insert {
                    continue;
                }

                let new_state = state.add_job(chain_index, allotment, compl);
                let is_new = known.insert(new_state.clone());
                if !is_new {
                    continue;
                }

                let tail = search(instance, chains, &new_state, known);
                if let Some(tail) = tail {
                    let mut path = Vec::with_capacity(tail.len() + 1);
                    let job = instance.jobs[new_job_index].clone();
                    let start_time = compl - job.processing_time(allotment);
                    path.push(ScheduledJob {
                        job,
                        allotment,
                        start_time,
                    });
                    path.extend(tail);
                    return Some(path);
                }
            }
        }
    }
    None
}

fn preprocess(instance: &Instance) -> Vec<Vec<usize>> {
    let mut chains: Vec<Vec<usize>> = vec![];
    for (job_index, job) in instance.jobs.iter().enumerate() {
        if let Some(chain) = chains.iter_mut().find(|chain| {
            chain
                .iter()
                .all(|&i| instance.jobs[i].is_comparable(&instance.constraints, job))
        }) {
            chain.push(job_index);
        } else {
            chains.push(vec![job_index]);
        }
    }
    for chain in &mut chains {
        chain.sort_by(|&left, &right| {
            match instance.jobs[left].compare(&instance.constraints, &instance.jobs[right]) {
                Some(true) => Ordering::Less,
                Some(false) => Ordering::Greater,
                _ => panic!("chain contains two non-comparable jobs"),
            }
        });
    }
    chains
}
