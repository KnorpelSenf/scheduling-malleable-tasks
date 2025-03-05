use good_lp::{constraint, default_solver, variable, variables, SolverModel};

use crate::algo::{Instance, Job, Schedule};

impl Job {
    fn work(&self, allotment: usize) -> i32 {
        allotment as i32 * self.processing_time(allotment)
    }
}

pub fn schedule(instance: Instance) -> Schedule {
    // initialization step
    // - compute rounding parameter rho
    let m = instance.jobs.len();
    let rho = compute_rho(m);
    // - compute allotment parameter µ
    let my = compute_my(m);

    // PHASE 1: linear program
    // - define linear program
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
    // TODO: add remaining constraints

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

fn compute_rho(m: usize) -> f64 {
    todo!()
}

fn compute_my(m: usize) -> f64 {
    todo!()
}
