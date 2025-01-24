/// A problem instance
pub struct Instance {
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
    fn processing_time(&self, allotment: usize) -> i32 {
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
    /// A list of scheduled jobs
    jobs: Vec<ScheduledJob>,
    // TODO: one vec per machine
}
/// A job that was scheduled in a feasible schedule
#[derive(Debug, Default)]
pub struct ScheduledJob {
    /// The input job
    job: Job,
    /// The job allotment
    allotment: usize,
    /// The integral starting time of the job
    start_time: i32,
}

#[derive(Clone, Debug, Default)]
struct State {
    ideal: Vec<Job>,
    allotment: Vec<usize>,
    completion_times: Vec<i32>,
}

pub fn schedule(instance: Instance) -> Schedule {
    Schedule { jobs: vec![] }
}
