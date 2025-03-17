use cpm_rs::{CustomTask, Scheduler};
use good_lp::{constraint, default_solver, variable, variables, SolverModel};

use crate::algo::{Instance, Job, PartialRelation, Schedule};

impl Job {
    fn work(&self, allotment: usize) -> i32 {
        allotment as i32 * self.processing_time(allotment)
    }
}

impl Instance {
    fn predecessors<'a>(&'a self, job: &Job) -> Vec<&'a Job> {
        self.jobs
            .iter()
            .filter(|j| j.less_than(&self.constraints, job))
            .collect()
    }
    fn successors<'a>(&'a self, job: &Job) -> Vec<&'a Job> {
        self.jobs
            .iter()
            .filter(|j| j.greater_than(&self.constraints, job))
            .collect()
    }
}

pub fn schedule(instance: Instance) -> Schedule {
    // initialization step
    let m = instance.jobs.len() as i32;
    // - compute rounding parameter rho
    let rho = compute_rho(m);
    // - compute allotment parameter µ
    let my = compute_my(m);

    // PHASE 1: linear program
    // - define linear program
    let l = critical_path_length(&instance);
    let w = instance
        .jobs
        .iter()
        .map(|job| job.processing_time(1))
        .sum::<i32>()
        / m;
    let mut vars = variables!();
    let makespan = vars.add(variable().min(0));
    let processing_times = instance
        .jobs
        .iter()
        .map(|_| vars.add(variable().min(0)))
        .collect::<Vec<_>>();
    let completion_times = instance
        .jobs
        .iter()
        .map(|_| vars.add(variable().min(0)))
        .collect::<Vec<_>>();
    // minimize makespan
    let problem = vars.minimise(makespan).using(default_solver);
    // set the makespan as the maximum completion time
    let problem = completion_times.iter().fold(problem, |problem, &c_j| {
        problem.with(constraint!(makespan >= c_j))
    });
    // TODO: add constraints

    // - obtain fractional solution
    let solution = problem
        .solve()
        .unwrap_or_else(|e| panic!("no solution: {e}"));
    // - round it to a feasible allotment

    // PHASE 2: list schedule
    // - generate new allotment
    // - run LIST to generate feasible schedule

    todo!("implement ILP schedule")
}

fn compute_rho(m: i32) -> f64 {
    todo!()
}

fn compute_my(m: i32) -> f64 {
    todo!()
}
fn critical_path_length(instance: &Instance) -> i32 {
    // TODO: the critical path depends on the processing times of the jobs,
    // which in turn depend on the allotments of each job, so perhaps we should
    // pick something else than allotment = 1 fixed as we do now?
    let mut scheduler = Scheduler::<i32>::new();
    for job in instance.jobs.iter() {
        scheduler
            .add_task(CustomTask::new(
                job.index.to_string(),
                job.processing_time(1),
                instance
                    .successors(job)
                    .iter()
                    .map(|job| job.index.to_string())
                    .collect(),
            ))
            .expect("duplicate task");
    }
    match scheduler.schedule() {
        Ok(()) => scheduler
            .get_critical_paths()
            .iter()
            .map(|path| path.get_dur())
            .max()
            .expect("empty graph"),
        Err(e) => panic!("{e}"),
    }
}
