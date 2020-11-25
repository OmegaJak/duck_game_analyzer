use plotters::prelude::*;

use chrono::{Duration, NaiveDateTime, Timelike};
use std::{error::Error, ops::Range};

pub fn plot_datetimes(datetimes: &Vec<NaiveDateTime>) -> Result<(), Box<dyn Error>> {
    let root =
        BitMapBackend::new("normal-dist.png", (2048, 1536)).into_drawing_area();

    root.fill(&WHITE)?;

    let x_values = datetimes.iter().map(|d| d.clone());
    let y_values = datetimes.iter().map(|d| get_fractional_time(&d));
    let points = x_values.zip(y_values).collect::<Vec<(NaiveDateTime, f64)>>();

    let x_range = Range::<NaiveDateTime> { start: datetimes.iter().min().unwrap().clone(), end: datetimes.iter().max().unwrap().clone() };
    let x_range = RangedDateTime::from(x_range);
    let horizontal_histogram_range = x_range.clone().step(Duration::weeks(1)).use_round().into_segmented();

    let y_range: Range<u32> = 0..24;
    let scatter_y_range: Range<f64> = 0.0..24.0;

    let areas = root.split_by_breakpoints([2000], [80]);

    let mut vertical_hist_ctx = ChartBuilder::on(&areas[0])
        .y_label_area_size(40)
        .build_cartesian_2d(horizontal_histogram_range, 0..50)?;
    let mut horizontal_hist_ctx = ChartBuilder::on(&areas[3])
        .x_label_area_size(40)
        .build_cartesian_2d(0..125, y_range.clone())?;
    let mut scatter_ctx = ChartBuilder::on(&areas[2])
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(x_range, scatter_y_range)?;
    scatter_ctx
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;
    scatter_ctx.draw_series(
        points
            .iter()
            .map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
    )?;
    let vertical_hist = Histogram::vertical(&vertical_hist_ctx)
        .style(GREEN.filled())
        .margin(0)
        .data(points.iter().map(|(x, _)| (*x, 1)));
    let horizontal_hist = Histogram::horizontal(&horizontal_hist_ctx)
        .style(GREEN.filled())
        .margin(0)
        .data(points.iter().map(|(_, y)| (*y as u32, 1)));
    vertical_hist_ctx.draw_series(vertical_hist)?;
    horizontal_hist_ctx.draw_series(horizontal_hist)?;

    Ok(())
}

fn get_fractional_time(datetime: &NaiveDateTime) -> f64 {
    let nanoseconds_of_second = datetime.nanosecond() as f64 / 1e9;
    let seconds_of_minute = datetime.second() as f64 / 60.0;
    let minutes_of_hour = datetime.minute() as f64 / 60.0;

    datetime.hour() as f64 + minutes_of_hour + seconds_of_minute + nanoseconds_of_second
}