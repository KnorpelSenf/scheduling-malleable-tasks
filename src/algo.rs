use std::{cmp::Ordering, hash::Hash};

/// A problem instance
#[derive(Debug)]
pub struct Instance {
    /// The number of processors available
    pub processor_count: usize,
    /// A list of jobs
    pub jobs: Vec<Job>,
    /// A partial ordering on the jobs
    pub constraints: Vec<Constraint>,
    /// The maximum number of seconds in the universe
    pub max_time: i32,
}
/// A job in a problem instance
#[derive(Clone, Debug, Default)]
pub struct Job {
    /// External identifier of the job
    pub id: i32,
    /// Index of the job
    pub index: usize,
    /// Processing times of the job based on how many machines is has available.
    /// Element 0 is skipped, so the vector starts with the processing time
    /// needed if the job is scheduled on one machine.
    pub processing_times: Vec<i32>,
}
impl Job {
    /// Computes the processing time of the job based on the given allotment
    pub fn processing_time(&self, allotment: usize) -> i32 {
        self.processing_times[allotment - 1]
    }
}
/// Compares two values by their index
#[derive(Debug)]
pub struct Constraint(pub usize, pub usize);
/// Implements a partial relation based on a list of constraints
trait PartialRelation {
    /// Returns `true` if self is comparable to other, and `false` of the two
    /// values are incomparable
    fn is_comparable(&self, relation: &Vec<Constraint>, other: &Self) -> bool {
        self.compare(relation, other).is_some()
    }
    /// Returns `true` if self is in relation to other, and `false` otherwise
    fn less_than(&self, relation: &Vec<Constraint>, other: &Self) -> bool {
        self.compare(relation, other).is_some_and(|less| less)
    }
    /// Returns `None` if self and other are incomparable. Returns `Some(true)`
    /// if self is less than other and returns `Some(false)` if other is less
    /// than self.
    fn compare(&self, relation: &Vec<Constraint>, other: &Self) -> Option<bool>;

    // /// Returns `true` if other is in relation to self, and `false` otherwise
    // fn greater_than(&self, relation: &Vec<Constraint>, other: &Self) -> bool {
    //     self.compare(relation, other).is_some_and(|less| !less)
    // }
}
impl PartialRelation for Job {
    fn compare(&self, relation: &Vec<Constraint>, other: &Self) -> Option<bool> {
        relation.iter().find_map(|&Constraint(left, right)| {
            if self.index == left && other.index == right {
                Some(true)
            } else if other.index == left && self.index == right {
                Some(false)
            } else {
                None
            }
        })
    }
}

/// A feasible job schedule
#[derive(Debug, Default)]
pub struct Schedule {
    /// The number of processors available
    pub processor_count: usize,
    /// A list of scheduled jobs
    pub jobs: Vec<ScheduledJob>,
}
/// A job that was scheduled in a feasible schedule
#[derive(Debug, Default)]
pub struct ScheduledJob {
    /// The input job
    pub job: Job,
    /// The job allotment
    pub allotment: usize,
    /// The integral starting time of the job
    pub start_time: i32,
}
impl ScheduledJob {
    /// Computes the processing time of the job based on the current allotment
    pub fn processing_time(&self) -> i32 {
        self.job.processing_time(self.allotment)
    }
}

#[derive(Clone, Debug)]
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
        State {
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
        State {
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

pub fn schedule(instance: Instance) -> Schedule {
    let chains = preprocess(&instance);
    let omega = chains.len();
    let initial_state = State::empty(omega);

    let path = search(&instance, &chains, initial_state).expect("no solution found");
    println!("path is now: {:#?}", path);
    // path contains a list of indices in which to add jobs in order to
    // reach the target state

    // TODO: convert path in graph to vector of scheduled jobs
    Schedule {
        processor_count: instance.processor_count,
        jobs: vec![],
    }
}

fn search(
    instance: &Instance,
    chains: &Vec<Vec<usize>>,
    state: State,
) -> Option<Vec<(usize, usize, i32)>> {
    // TODO: look this up in some state table in order to avoid searching
    // the entire tree

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
                // TODO: iterate completion times, check the number of available
                // processors at each time

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

                let tail = search(instance, chains, new_state);
                if let Some(tail) = tail {
                    let mut path = Vec::with_capacity(tail.len() + 1);
                    path.push((new_job_index, allotment, compl));
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
    for chain in chains.iter_mut() {
        chain.sort_by(|&left, &right| {
            match instance.jobs[left].compare(&instance.constraints, &instance.jobs[right]) {
                Some(true) => Ordering::Less,
                Some(false) => Ordering::Greater,
                _ => panic!("welp"),
            }
        });
    }
    chains
}
