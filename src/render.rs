use std::cmp::max;

use crate::{Schedule, ScheduledJob};
use svg::{
    node::element::{
        path::Data, Group, LinearGradient, Path, Rectangle, Stop, Style, Text, Title, SVG,
    },
    Document,
};

const SCALE_MARGIN: usize = 50; // px
const LEFT_MARGIN: usize = SCALE_MARGIN + 20; // px
const TOP_HEADER_MARGIN: usize = 50; // px
const TOP_MARGIN: usize = TOP_HEADER_MARGIN + 20; // px
const RIGHT_MARGIN: usize = 30; // px
const BOTTOM_MARGIN: usize = 20; // px
const MACHINE_WIDTH: usize = 150; // px
const MACHINE_HEIGHT_SCALE: usize = 15; // px for each unit of processing time
const MACHINE_SPACING: usize = 10; // px

pub fn render_schedule(schedule: Schedule) -> String {
    // Create the linear gradient for the background
    let gradient = LinearGradient::new()
        .set("id", "background")
        .set("y1", "0")
        .set("y2", "1")
        .set("x1", "0")
        .set("x2", "0")
        .add(Stop::new().set("stop-color", "#eeeeee").set("offset", "5%"))
        .add(
            Stop::new()
                .set("stop-color", "#b0b0ee")
                .set("offset", "95%"),
        );

    let document = (0..schedule.processor_count)
        .map(create_machine_header)
        .fold(
            Document::new()
                .set("version", "1.1")
                .set("xmlns", "http://www.w3.org/2000/svg")
                .set("xmlns:svg", "http://www.w3.org/2000/svg")
                .add(gradient)
                .add(Style::new(
                    r#"
    text { font-family:monospace; font-size:10px; fill:black; }
    #title { text-anchor:middle; font-size:25px; }
    .machine-header { text-anchor:middle; font-size:17px; }
    .machine-box { stroke-width:1; stroke:black; }
    .machine-label { text-anchor:middle; dominant-baseline:middle; font-size:15px; }
    .scale-label { text-anchor:end; dominant-baseline:middle; font-size:10px; }
    "#,
                ))
                // background
                .add(
                    Rectangle::new()
                        .set("x", 0)
                        .set("y", 0)
                        .set("width", "100%")
                        .set("height", "100%")
                        .set("fill", "url(#background)"),
                )
                // title
                .add(
                    Text::new("Schedule")
                        .set("id", "title")
                        .set("x", "50%")
                        .set("y", 24),
                ),
            |doc, header| doc.add(header),
        );

    // Create the SVG document
    let height = schedule
        .jobs
        .iter()
        .map(|job| job.start_time + job.processing_time())
        .max()
        .unwrap_or(0) as usize;
    let body = add_jobs_to_doc(document, schedule.processor_count, schedule.jobs)
        .add(create_time_scale(height))
        .set(
            "width",
            LEFT_MARGIN + schedule.processor_count * (MACHINE_WIDTH + MACHINE_SPACING)
                - MACHINE_SPACING
                + RIGHT_MARGIN,
        )
        .set(
            "height",
            TOP_MARGIN + height * MACHINE_HEIGHT_SCALE + BOTTOM_MARGIN,
        )
        .to_string();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
{body}"#
    )
}

fn add_jobs_to_doc(document: SVG, processor_count: usize, mut jobs: Vec<ScheduledJob>) -> SVG {
    jobs.sort_by_key(|job| job.start_time);
    jobs.into_iter()
        .fold(
            (document, vec![0; processor_count]),
            |(doc, mut used_until), job| {
                let y = TOP_MARGIN + job.start_time as usize * MACHINE_HEIGHT_SCALE;
                let processors: Vec<usize> = used_until
                    .iter()
                    .enumerate()
                    .filter(|(_, used)| **used <= job.start_time)
                    .take(job.allotment)
                    .map(|(proc, _)| proc)
                    .collect();
                assert_eq!(
                    processors.len(),
                    job.allotment,
                    "insufficient number of processors available"
                );
                let end = job.start_time + job.processing_time();
                for proc in processors.iter() {
                    used_until[*proc] = end;
                }
                (add_job_to_doc(doc, processors, y, job), used_until)
            },
        )
        .0
}

fn add_job_to_doc(document: SVG, processors: Vec<usize>, y: usize, job: ScheduledJob) -> SVG {
    assert_eq!(
        processors.len(),
        job.allotment,
        "mismatched number of processors for allotment"
    );
    let processing_time = job.processing_time() as usize;
    let w = MACHINE_WIDTH;
    let h = MACHINE_HEIGHT_SCALE * processing_time;
    processors.into_iter().fold(document, |doc, processor| {
        let x = LEFT_MARGIN + processor * (MACHINE_WIDTH + MACHINE_SPACING);
        let machine_box = Rectangle::new()
            .set("x", x)
            .set("y", y)
            .set("width", w)
            .set("height", h)
            .set("fill", "#0000f8")
            .set("class", "machine-box");

        let machine_label = Text::new(job.job.id.to_string())
            .set("x", x + w / 2) // Centered on the rectangle
            .set("y", y + h / 2)
            .set("class", "machine-label");

        let tooltip = Title::new(format!(
            "Job {}\n\nallotment: {} processors\nprocessing time: {} s",
            job.job.id, job.allotment, processing_time
        ));

        let group = Group::new()
            .add(machine_box)
            .add(machine_label)
            .add(tooltip);

        doc.add(group)
    })
}

fn create_machine_header(i: usize) -> Text {
    let x = LEFT_MARGIN + i * (MACHINE_WIDTH + MACHINE_SPACING) + (MACHINE_WIDTH / 2);
    let y = TOP_HEADER_MARGIN;
    Text::new(format!("Processor {i}"))
        .set("x", x)
        .set("y", y)
        .set("width", "100%")
        .set("height", "100%")
        .set("class", "machine-header")
}

fn create_time_scale(height_seconds: usize) -> Group {
    (0..=height_seconds)
        .map(|t| {
            let scaled_t = t * MACHINE_HEIGHT_SCALE;
            let is_big = scaled_t % (5 * MACHINE_HEIGHT_SCALE) == 0;
            let width = if is_big { 10 } else { 5 };
            let line = Group::new().add(create_line(
                SCALE_MARGIN - width,
                TOP_MARGIN + scaled_t,
                width,
                0,
            ));
            if is_big || t == height_seconds {
                line.add(
                    Text::new(t.to_string())
                        .set("x", SCALE_MARGIN - 15)
                        .set("y", TOP_MARGIN + scaled_t)
                        .set("class", "scale-label"),
                )
            } else {
                line
            }
        })
        .fold(
            Group::new().add(create_line(
                SCALE_MARGIN,
                TOP_MARGIN,
                0,
                height_seconds * MACHINE_HEIGHT_SCALE,
            )),
            |group, line| group.add(line),
        )
}

fn create_line(x: usize, y: usize, w: usize, h: usize) -> Path {
    Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 2)
        .set("d", Data::new().move_to((x, y)).line_by((w, h)))
}
