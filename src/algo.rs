/// A problem instance
pub struct Instance {
    processor_count: usize,
    /// A list of jobs
    jobs: Vec<Job>,
    /// A partial ordering on the jobs
    constraints: Vec<Constraint>,
}
pub struct Job {
    /// Index of the job
    index: usize,
    /// Processing times of the job based on how many machines is has available
    processing_time: Vec<i32>,
}
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
/// Compares two values by their index
pub struct Constraint(usize, usize);

/// A feasible job schedule
pub struct Schedule {
    /// A list of scheduled jobs
    jobs: Vec<ScheduledJob>,
}
/// A job that was scheduled in a feasible schedule
pub struct ScheduledJob {
    /// The input job
    job: Job,
    /// The job allotment
    allotment: usize,
    /// The integral starting time of the job
    start_time: i32,
}

fn schedule(instance: Instance) -> Schedule {
    Schedule { jobs: vec![] }
}
