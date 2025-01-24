use std::cmp::max;

use crate::{Schedule, ScheduledJob};
use svg::{
    node::element::{path::Data, Group, LinearGradient, Path, Rectangle, Stop, Style, Text, SVG},
    Document,
};

const SCALE_MARGIN: usize = 50; // px
const LEFT_MARGIN: usize = SCALE_MARGIN + 20; // px
const TOP_HEADER_MARGIN: usize = 50; // px
const TOP_MARGIN: usize = TOP_HEADER_MARGIN + 20; // px
const RIGHT_MARGIN: usize = 30; // px
const BOTTOM_MARGIN: usize = 20; // px
const MACHINE_WIDTH: usize = 100; // px
const MACHINE_HEIGHT_SCALE: usize = 15; // px for each unit of processing time
const MACHINE_SPACING: usize = 10; // px

pub fn render_schedule(schedule: &Schedule) -> String {
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
    let (document, height) = add_jobs_to_doc(document, &schedule.jobs);

    let body = document
        .add(create_time_scale(height))
        .set(
            "width",
            LEFT_MARGIN + schedule.processor_count * (MACHINE_WIDTH + MACHINE_SPACING)
                - MACHINE_SPACING
                + RIGHT_MARGIN,
        )
        .set("height", height + BOTTOM_MARGIN)
        .to_string();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
{body}"#
    )
}

fn add_jobs_to_doc(document: SVG, jobs: &Vec<ScheduledJob>) -> (SVG, usize) {
    jobs.iter().enumerate().fold(
        (document, TOP_MARGIN),
        |(doc, max_height), (machine, job)| {
            let x = LEFT_MARGIN + machine * (MACHINE_WIDTH + MACHINE_SPACING);
            let y = TOP_MARGIN + job.start_time as usize;
            // let (svg, height) = add_job_to_doc(doc, x, y, job);
            (doc, max(0, max_height))
        },
    )
}

fn create_machine_header(i: usize) -> Text {
    let x = LEFT_MARGIN + i * (MACHINE_WIDTH + MACHINE_SPACING) + (MACHINE_WIDTH / 2);
    let y = TOP_HEADER_MARGIN;
    Text::new(format!("Machine {i}"))
        .set("x", x)
        .set("y", y)
        .set("width", "100%")
        .set("height", "100%")
        .set("class", "machine-header")
}

fn create_time_scale(height: usize) -> Group {
    (0..height / MACHINE_HEIGHT_SCALE)
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
            if is_big {
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
                height - TOP_MARGIN,
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
