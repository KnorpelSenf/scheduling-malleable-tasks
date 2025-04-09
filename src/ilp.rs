use cpm_rs::{CustomTask, Scheduler};
use good_lp::{constraint, default_solver, variable, variables, Expression, Solution, SolverModel};

use crate::algo::{Instance, Job, PartialRelation, Schedule};

// impl Job {
//     fn work(&self, allotment: usize) -> i32 {
//         allotment as i32 * self.processing_time(allotment)
//     }
// }

impl Instance {
    fn predecessors<'a>(&'a self, job: &Job) -> Vec<(usize, &'a Job)> {
        self.jobs
            .iter()
            .enumerate()
            .filter(|(_, j)| job.index != j.index && j.less_than(&self.constraints, job))
            .collect()
    }
    fn successors<'a>(&'a self, job: &Job) -> Vec<(usize, &'a Job)> {
        self.jobs
            .iter()
            .enumerate()
            .filter(|(_, j)| job.index != j.index && j.greater_than(&self.constraints, job))
            .collect()
    }
}

pub fn schedule(instance: Instance) -> Schedule {
    // initialization step
    let m = instance.jobs.len() as i32;
    // - compute rounding parameter rho
    // let rho = compute_rho(m);
    // - compute allotment parameter µ
    // let my = compute_my(m);

    // PHASE 1: linear program
    // - define linear program
    let cpl = critical_path_length(&instance);
    let mut vars = variables!();
    let makespan = vars.add(variable().min(0));
    let processing_times = instance
        .jobs
        .iter()
        .map(|job| {
            vars.add(variable().clamp(
                job.processing_time(instance.processor_count),
                job.processing_time(1),
            ))
        })
        .collect::<Vec<_>>();
    let completion_times = instance
        .jobs
        .iter()
        .map(|_| vars.add(variable().clamp(0, cpl)))
        .collect::<Vec<_>>();
    let work = instance
        .jobs
        .iter()
        .map(|_| vars.add(variable()))
        .collect::<Vec<_>>();
    // minimize makespan
    let problem = vars.minimise(makespan).using(default_solver);
    // set the makespan as the maximum completion time
    let problem = completion_times.iter().fold(problem, |prob, &c_j| {
        prob.with(constraint!(makespan >= c_j))
    });
    // ensure the order of jobs
    let problem = instance
        .jobs
        .iter()
        .enumerate()
        .fold(problem, |prob, (i, job)| {
            instance
                .predecessors(job)
                .into_iter()
                .fold(prob, |p, (j, _)| {
                    p.with(constraint!(
                        completion_times[i] + processing_times[j] <= completion_times[j]
                    ))
                })
        });
    // (9)
    let problem = (1..=instance.processor_count - 1).fold(problem, |prob, l| {
        (0..m as usize).fold(prob, |p, j| {
            let job = &instance.jobs[j];
            let p_j_l = job.processing_time(l);
            let p_j_lp1 = job.processing_time(l + 1);
            let l = l as i32;
            let lp1 = l + 1;
            let r = (lp1 * p_j_lp1 - l * p_j_l) / (p_j_lp1 - p_j_l);
            let s = (p_j_l * p_j_lp1) / (p_j_lp1 - p_j_l);
            p.with(constraint!(r * processing_times[j] - s <= work[j]))
        })
    });
    let problem = problem.with(constraint!(work.iter().sum::<Expression>() / m <= makespan));

    // - obtain fractional solution
    let solution = problem
        .solve()
        .unwrap_or_else(|e| panic!("no solution: {e}"));
    for (i, x_j) in processing_times.into_iter().enumerate() {
        println!("x_{i} = {}", solution.value(x_j));
    }
    for (i, c_j) in completion_times.into_iter().enumerate() {
        println!("C_{i} = {}", solution.value(c_j));
    }
    for (i, w_j) in work.into_iter().enumerate() {
        println!("w_{i} = {}", solution.value(w_j));
    }
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
    let mut scheduler = Scheduler::<i32>::new();
    for job in instance.jobs.iter() {
        scheduler
            .add_task(CustomTask::new(
                job.index.to_string(),
                job.processing_time(1),
                instance
                    .successors(job)
                    .iter()
                    .map(|(_, job)| job.index.to_string())
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
