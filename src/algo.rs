use itertools::{enumerate, Itertools};
use std::collections::{HashMap, LinkedList};

/// A problem instance
pub struct Instance {
    /// The number of processors available
    pub processor_count: usize,
    /// A list of jobs
    pub jobs: Vec<Job>,
    /// A partial ordering on the jobs
    pub constraints: Vec<Constraint>,
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
    /// Returns `true` if other is in relation to self, and `false` otherwise
    fn greater_than(&self, relation: &Vec<Constraint>, other: &Self) -> bool {
        self.compare(relation, other).is_some_and(|less| !less)
    }
    /// Returns `None` if self and other are incomparable. Returns `Some(true)`
    /// if self is less than other and returns `Some(false)` if other is less
    /// than self.
    fn compare(&self, relation: &Vec<Constraint>, other: &Self) -> Option<bool>;
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
struct State {
    len: usize,
    ideal: Vec<Option<Job>>,
    allotment: Vec<usize>,
    completion_times: Vec<i32>,
}
impl State {
    fn empty(omega: usize) -> Self {
        State {
            len: omega,
            ideal: vec![None; omega],
            allotment: vec![0; omega],
            completion_times: vec![0; omega],
        }
    }
    fn start_times(&self, i: usize) -> i32 {
        self.ideal[i]
            .as_ref()
            .map(|ideal| self.completion_times[i] - ideal.processing_time(self.allotment[i]))
            .unwrap_or(0)
    }
    fn is_valid(&self) -> bool {
        // TODO: iterate completion times, check the number of available
        // processors at each time
        true
    }
    // fn try_add_job(&self, i: usize, job: Job) -> Option<Self> { None }
    // fn can_add(&self, i: usize, job: Job) -> bool { false }
    fn add_job(&self, i: usize, job: &Job) -> Self {
        let len = self.len;
        let mut ideal = self.ideal.clone();
        let allotment = self.allotment.clone();
        let completion_times = self.completion_times.clone();
        ideal[i] = Some(job.clone());
        State {
            len,
            ideal,
            allotment,
            completion_times,
        }
    }
}

pub fn schedule(instance: Instance) -> Schedule {
    let chains = preprocess(&instance);
    let omega = chains.len();
    let initial_state = State::empty(omega);

    let path = search(&instance, initial_state).expect("no solution found");
    // path contains a list of indices in which to add jobs in order to
    // reach the target state

    // TODO: convert path in graph to vector of scheduled jobs
    Schedule {
        processor_count: instance.processor_count,
        jobs: vec![],
    }
}

fn search(instance: &Instance, state: State) -> Option<Vec<usize>> {
    if !state.is_valid() {
        return None;
    };
    for i in 0..state.len {
        // TODO: look this up in some state table in order to avoid searching
        // the entire tree
        let new_state = state.add_job(i, &instance.jobs[i]);
        let tail = search(instance, new_state);
        if let Some(tail) = tail {
            let mut path = Vec::with_capacity(tail.len() + 1);
            path.push(i);
            path.extend(tail);
            return Some(path);
        }
    }
    None
}

fn preprocess(instance: &Instance) -> Vec<Vec<usize>> {
    let mut chains = (0..instance.jobs.len())
        .map(|i| vec![i])
        .collect::<Vec<_>>();
    for Constraint(l, r) in instance.constraints.iter() {
        let (i, left) = chains
            .iter()
            .enumerate()
            .find(|(_, chain)| chain.iter().contains(l))
            .expect("bad constraint");
        let mut left = left.clone();
        let (j, right) = chains
            .iter()
            .enumerate()
            .find(|(_, chain)| chain.iter().contains(r))
            .expect("bad constraint");
        let mut right = right.clone();
        right.append(&mut left);
        chains[i] = vec![];
        chains[j] = right;
    }
    chains

    // TODO: fix the following attempt at writing a faster impl
    // let mut chains: Vec<LinkedList<usize>> = vec![];
    // let mut mapping: Vec<usize> = vec![];
    // let mut index: HashMap<usize, usize> = HashMap::new();
    // for &Constraint(l, r) in instance.constraints.iter() {
    //     // Need to hash twice because we cannot borrow mut twice
    //     let has_left = index.contains_key(&l);
    //     let has_right = index.contains_key(&r);

    //     if has_left && has_right {
    //         // merge chains left and right containing l and r
    //         let left_index = *index.get(&l).expect("bad check");
    //         let right_index = *index.get(&r).expect("bad check");
    //         let appendix = &mut mapping
    //             .get(left_index)
    //             .and_then(|&i| chains.get(i))
    //             .expect("bad index")
    //             .clone();
    //         mapping
    //             .get(right_index)
    //             .and_then(|&i| chains.get_mut(i))
    //             .expect("bad index")
    //             .append(appendix);
    //         mapping[left_index] = mapping[right_index];
    //     } else if has_right {
    //         // add l to chain
    //         let right = *index.get(&r).expect("bad check");
    //         mapping
    //             .get(right)
    //             .and_then(|&i| chains.get_mut(i))
    //             .expect("bad index")
    //             .push_back(l);
    //         index.insert(l, right);
    //     } else if has_left {
    //         // add r to chain
    //         let left = *index.get(&l).expect("bad check");
    //         mapping
    //             .get(left)
    //             .and_then(|&i| chains.get_mut(i))
    //             .expect("bad index")
    //             .push_back(r);
    //         index.insert(r, left);
    //     } else {
    //         // create a new chain with l and r
    //         let mut chain = LinkedList::new();
    //         chain.push_back(l);
    //         chain.push_back(r);
    //         let i = chains.len();
    //         chains.push(chain);
    //         let j = mapping.len();
    //         mapping.push(i);
    //         index.insert(l, j);
    //         index.insert(r, j);
    //     }
    // }
    // index
    //     .values()
    //     .unique()
    //     .filter_map(|&i| mapping.get(i))
    //     .filter_map(|&i| chains.get(i))
    //     .map(|list| list.iter().copied().collect::<Vec<_>>())
    //     .collect::<Vec<_>>()
}
