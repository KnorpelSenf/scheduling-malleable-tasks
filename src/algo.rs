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
impl Instance {
    pub fn predecessors<'a>(&'a self, job: &Job) -> Vec<(usize, &'a Job)> {
        self.jobs
            .iter()
            .enumerate()
            .filter(|(_, j)| job.index != j.index && j.less_than(&self.constraints, job))
            .collect()
    }
    pub fn successors<'a>(&'a self, job: &Job) -> Vec<(usize, &'a Job)> {
        self.jobs
            .iter()
            .enumerate()
            .filter(|(_, j)| job.index != j.index && j.greater_than(&self.constraints, job))
            .collect()
    }
}
/// A job in a problem instance
#[derive(Clone, Debug)]
pub struct Job {
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
    pub fn closest_allotment(&self, processing_time: i32) -> usize {
        1 + self
            .processing_times
            .iter()
            .copied()
            .map(|x| processing_time.abs_diff(x))
            .enumerate()
            .min_by_key(|&(_, diff)| diff)
            .expect("no processing times")
            .0
    }
}
impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
// impl Eq for Job {}
/// Compares two values by their index
#[derive(Debug)]
pub struct Constraint(pub usize, pub usize);
/// Implements a partial relation based on a list of constraints
pub trait PartialRelation {
    /// Returns `true` if self is comparable to other, and `false` of the two
    /// values are incomparable
    fn is_comparable(&self, relation: &[Constraint], other: &Self) -> bool {
        self.compare(relation, other).is_some()
    }
    /// Returns `true` if self is in relation to other, and `false` otherwise
    fn less_than(&self, relation: &[Constraint], other: &Self) -> bool {
        self.compare(relation, other).is_some_and(|less| less)
    }
    // /// Returns `true` if other is in relation to self, and `false` otherwise
    fn greater_than(&self, relation: &[Constraint], other: &Self) -> bool {
        self.compare(relation, other).is_some_and(|less| !less)
    }
    /// Returns `None` if self and other are incomparable. Returns `Some(true)`
    /// if self is less than other and returns `Some(false)` if other is less
    /// than self.
    fn compare(&self, relation: &[Constraint], other: &Self) -> Option<bool>;
}
impl PartialRelation for Job {
    fn compare(&self, relation: &[Constraint], other: &Self) -> Option<bool> {
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
#[derive(Debug)]
pub struct Schedule {
    /// The number of processors available
    pub processor_count: usize,
    /// A list of scheduled jobs
    pub jobs: Vec<ScheduledJob>,
}
/// A job that was scheduled in a feasible schedule
#[derive(Debug)]
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
    /// Computes the completion time of the job based on the current allotment
    pub fn completion_time(&self) -> i32 {
        self.start_time + self.processing_time()
    }
}
