use cpm_rs::{CustomTask, Scheduler};
use good_lp::{
    constraint, default_solver, variable, variables, Expression, Solution, SolverModel, Variable,
};

use crate::algo::{Instance, Job, Schedule, ScheduledJob};

pub fn schedule(instance: Instance, compress: bool) -> Schedule {
    // initialization step
    let m = instance.processor_count;
    let rho = compute_rho(m);

    // PHASE 1: linear program
    // - define linear program
    let cpl = critical_path_length(&instance);
    let total_processing_time = instance
        .jobs
        .iter()
        .map(|job| job.processing_time(1))
        .sum::<i32>();
    let mut vars = variables!();
    let makespan = vars.add(variable().min(0));
    let total_work = vars.add(variable().min(0));
    let completion_times = instance
        .jobs
        .iter()
        .map(|_| vars.add(variable().clamp(0, cpl)))
        .collect::<Vec<_>>();
    let processing_times = instance
        .jobs
        .iter()
        .map(|job| vars.add(variable().clamp(0, job.processing_time(1))))
        .collect::<Vec<_>>();
    let virtual_processing_times = instance
        .jobs
        .iter()
        .map(|_| {
            (0..m)
                .map(|_| vars.add(variable().min(0)))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let problem = vars.minimise(makespan).using(default_solver);

    let problem = instance
        .jobs
        .iter()
        .enumerate()
        .fold(problem, |prob, (j, job)| {
            instance
                .successors(job)
                .into_iter()
                .fold(prob, |p, (k, _)| {
                    p.with(constraint!(
                        completion_times[j] + processing_times[k] <= completion_times[k]
                    ))
                })
        });
    let problem = instance
        .jobs
        .iter()
        .enumerate()
        .fold(problem, |prob, (j, _)| {
            (0..m).fold(prob, |p, i| {
                p.with(constraint!(
                    virtual_processing_times[j][i] <= processing_times[j]
                ))
            })
        });
    let problem = instance
        .jobs
        .iter()
        .enumerate()
        .fold(problem, |prob, (j, job)| {
            (1..m).fold(prob, |p, i| {
                p.with(constraint!(
                    virtual_processing_times[j][i] <= job.processing_time(i)
                ))
            })
        });
    let problem = instance
        .jobs
        .iter()
        .enumerate()
        .fold(problem, |prob, (j, job)| {
            prob.with(constraint!(
                virtual_processing_times[j][m - 1] == job.processing_time(m)
            ))
        });
    let problem = problem
        .with(constraint!(
            instance
                .jobs
                .iter()
                .enumerate()
                .map(|(j, job)| w_hat_j(m, &virtual_processing_times[j], job))
                .sum::<Expression>()
                + total_processing_time
                <= total_work
        ))
        .with(constraint!(cpl <= makespan))
        .with(constraint!(total_work / (m as i32) <= makespan));

    // - obtain fractional solution
    let solution = problem
        .solve()
        .unwrap_or_else(|e| panic!("no solution: {e}"));

    println!("Believe makespan to be {}", solution.value(makespan));

    let completion_times = completion_times
        .into_iter()
        .map(|v| solution.value(v).round() as i32)
        .collect::<Vec<_>>();

    for (i, c_j) in completion_times.iter().copied().enumerate() {
        println!("C_{i} = {}", c_j);
    }

    let allotments = virtual_processing_times
        .into_iter()
        .enumerate()
        .map(|(j, vec)| {
            vec.into_iter()
                .zip(1..=m)
                .map(|(var, i)| {
                    let val = solution.value(var);
                    println!("x_{j}_{i} = {val}");
                    let p_j_i = instance.jobs[j].processing_time(i);
                    if val < p_j_i as f64 * rho {
                        (i, 0)
                    } else {
                        (i, p_j_i)
                    }
                })
                .max_by_key(|&(_, p)| p)
                .map(|(i, _)| i)
                .unwrap_or(0)
        })
        .collect::<Vec<_>>();

    for (i, l_j) in allotments.iter().copied().enumerate() {
        println!("l_{i} = {}", l_j);
    }

    // PHASE 2: list schedule
    // - run LIST to generate feasible schedule
    let mut jobs = (0..instance.jobs.len())
        .map(|i| (i, true))
        .collect::<Vec<_>>();
    let mut scheduled_jobs: Vec<ScheduledJob> = vec![];
    let mut occupation = vec![0; instance.processor_count];
    for _ in 0..jobs.len() {
        // find READY jobs
        let (pick, start_time) = jobs
            .iter()
            .filter(|(_, available)| *available)
            .filter_map(|&(job, _)| {
                instance
                    .predecessors(&instance.jobs[job])
                    .iter()
                    .map(|(_, p)| scheduled_jobs.iter().find(|s| s.job.index == p.index))
                    .collect::<Option<Vec<_>>>()
                    .map(|s| (job, s))
            })
            .map(|(job, scheduled_predecessors)| {
                let allotment = allotments[job];
                let starting_time = if compress {
                    0
                } else {
                    completion_times[job] - instance.jobs[job].processing_time(allotment)
                };

                let predecessors_finished_at = scheduled_predecessors
                    .iter()
                    .map(|s| s.completion_time())
                    .max()
                    .unwrap_or(0);

                let fit = occupation[occupation.len() - allotment];

                let earliest = starting_time.max(predecessors_finished_at).max(fit);

                (job, earliest)
            })
            // take min by starting time
            .min_by_key(|&(_, alpha)| alpha)
            .expect("no job ready");
        jobs[pick].1 = false;
        let allotment = allotments[pick];
        let job = ScheduledJob {
            job: instance.jobs[pick].clone(),
            allotment,
            start_time,
        };
        // update occupation
        let machine = occupation
            .iter()
            .enumerate()
            .find(|(_, o)| **o <= start_time)
            .expect("bad start time")
            .0;
        let done = job.completion_time();
        for i in machine..machine + allotment {
            occupation[i] = done;
        }
        scheduled_jobs.push(job);
    }
    Schedule {
        processor_count: instance.processor_count,
        jobs: scheduled_jobs,
    }
}

fn w_hat_j(m: usize, virtual_processing_times: &Vec<Variable>, job: &Job) -> Expression {
    (1..=m)
        .map(|i| w_bar_j_i(m, i, virtual_processing_times, job))
        .sum::<Expression>()
}
fn w_bar_j_i(
    m: usize,
    i: usize,
    virtual_processing_times: &Vec<Variable>,
    job: &Job,
) -> Expression {
    if i == m {
        0.into()
    } else {
        (w_j_l(i + 1, job) - w_j_l(i, job)) * (job.processing_time(i) - virtual_processing_times[i])
            / job.processing_time(i)
    }
}
fn w_j_l(allotment: usize, job: &Job) -> i32 {
    allotment as i32 * job.processing_time(allotment)
}

fn compute_rho(_m: usize) -> f64 {
    0.430991
}
// fn compute_my(m: usize) -> f64 {
//     0.270875 * m as f64
// }

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
