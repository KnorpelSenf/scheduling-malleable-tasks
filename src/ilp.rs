use cpm_rs::{CustomTask, Scheduler};
use good_lp::{constraint, default_solver, variable, variables, Expression, Solution, SolverModel};

use crate::algo::{Instance, Job, PartialRelation, Schedule, ScheduledJob};

impl Job {
    fn closest_allotment(&self, processing_time: i32) -> usize {
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
    let processing_times = processing_times
        .into_iter()
        .map(|v| solution.value(v).round() as i32)
        .collect::<Vec<_>>();
    let completion_times = completion_times
        .into_iter()
        .map(|v| solution.value(v).round() as i32)
        .collect::<Vec<_>>();

    for (i, x_j) in processing_times.iter().copied().enumerate() {
        // print solution
        println!("x_{i} = {}", x_j);
    }
    for (i, c_j) in completion_times.iter().copied().enumerate() {
        println!("C_{i} = {}", c_j);
    }
    // - round it to a feasible allotment
    // - compute allotment parameter µ
    let my = compute_my(m).floor() as usize;
    let allotments = processing_times
        .iter()
        .copied()
        .zip(instance.jobs.iter())
        .map(|(x_j, job)| job.closest_allotment(x_j).min(my))
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
                let starting_time =
                    completion_times[job] - instance.jobs[job].processing_time(allotment);

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

fn compute_my(m: i32) -> f64 {
    let m = m as f64;
    0.01 * (113.0 * m - ((6469.0 * m * m) - 6300.0 * m).sqrt())
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
