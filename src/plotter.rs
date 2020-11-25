use plotters::{coord, prelude::*};

use chrono::{Duration, NaiveDateTime, Timelike};
use std::{error::Error, ops::Range};

pub fn plot_datetimes(datetimes: &Vec<NaiveDateTime>) -> Result<(), Box<dyn Error>> {
	let image_dimensions = ImageDimensions { width: 2500, height: 1200 };
    let root = BitMapBackend::new("playtimes-dist.png", <(u32, u32)>::from(&image_dimensions)).into_drawing_area();

    root.fill(&WHITE)?;

    let x_values = datetimes.iter().map(|d| d.clone());
    let y_values = datetimes.iter().map(|d| get_fractional_time(&d));
    let points = x_values.zip(y_values).collect::<Vec<(NaiveDateTime, f64)>>();

    let x_range = Range::<NaiveDateTime> { start: datetimes.iter().min().unwrap().clone() - Duration::weeks(1), end: datetimes.iter().max().unwrap().clone() + Duration::weeks(1) };
    let x_range = RangedDateTime::from(x_range);
    let horizontal_histogram_range = x_range.clone().step(Duration::weeks(1)).use_floor().into_segmented();

    let y_range: Range<u32> = 0..24;
    let scatter_y_range: Range<f64> = 0.0..24.0;

	let areas = get_areas(root);

	let top_hist_max_count = 50;
	let right_hist_max_count = 125;
	let scatter_x_label_height = 35;
	let scatter_y_label_width = 70;
    let mut top_hist_ctx = ChartBuilder::on(&areas[0])
        .y_label_area_size(scatter_y_label_width)
		.caption("5 Years of Duck Game Podiums", FontDesc::new(FontFamily::SansSerif, 40.0, FontStyle::Bold))
        .build_cartesian_2d(horizontal_histogram_range, 0..top_hist_max_count)?;
    let mut right_hist_ctx = ChartBuilder::on(&areas[3])
		.x_label_area_size(scatter_x_label_height)
        .build_cartesian_2d(0..right_hist_max_count, y_range.clone())?;
    let mut scatter_ctx = ChartBuilder::on(&areas[2])
		
		.x_label_area_size(scatter_x_label_height)
		.y_label_area_size(scatter_y_label_width)
		.build_cartesian_2d(x_range, scatter_y_range)?;
    scatter_ctx
        .configure_mesh()
		.x_labels(31)
		.x_label_formatter(&|asdf| format_date(asdf))
		.x_label_style(FontDesc::new(FontFamily::SansSerif, 20.0, FontStyle::Normal))
		.y_labels(24)
		.y_label_formatter(&|y| format_hour(*y))
		.y_label_style(FontDesc::new(FontFamily::SansSerif, 20.0, FontStyle::Normal))
		.light_line_style(WHITE.filled())
        .draw()?;
    scatter_ctx.draw_series(
        points
            .iter()
            .map(|(x, y)| Circle::new((*x, *y), 3, GREEN.filled())),
    )?;
    let top_hist = Histogram::vertical(&top_hist_ctx)
        .style(GREEN.filled())
        .margin(0)
        .data(points.iter().map(|(x, _)| (*x, 1)));
    let right_hist = Histogram::horizontal(&right_hist_ctx)
        .style(GREEN.filled())
        .margin(0)
        .data(points.iter().map(|(_, y)| (*y as u32, 1)));
    top_hist_ctx.draw_series(top_hist)?;
    right_hist_ctx.draw_series(right_hist)?;

    Ok(())
}

fn get_fractional_time(datetime: &NaiveDateTime) -> f64 {
    let nanoseconds_of_second = datetime.nanosecond() as f64 / 1e9;
    let seconds_of_minute = datetime.second() as f64 / 60.0;
    let minutes_of_hour = datetime.minute() as f64 / 60.0;

    datetime.hour() as f64 + minutes_of_hour + seconds_of_minute + nanoseconds_of_second
}

fn format_hour(hour: f64) -> String {
	if hour > 0.0 && hour < 12.0 {
		format!("{:.0} AM", hour)
	} else {
		format!("{:.0} PM", hour as u32 % 12)
	}
}

fn format_date(datetime: &NaiveDateTime) -> String {
	format!("{}", datetime.format("%b '%y"))
}

fn get_areas(root: DrawingArea<BitMapBackend, coord::Shift>) -> Vec<DrawingArea<BitMapBackend, coord::Shift>> {
	let top_histogram_vertical_fraction = 0.13;
	let main_chart_horizontal_fraction = 0.92;

	let top_histogram_pixel_height = root.relative_to_height(top_histogram_vertical_fraction) as u32;
	let scatter_pixel_width = root.relative_to_width(main_chart_horizontal_fraction) as u32;
	let areas = root.split_by_breakpoints([scatter_pixel_width], [top_histogram_pixel_height]);

	areas
}

struct ImageDimensions {
	pub width: u32,
	pub height: u32
}

impl From<&ImageDimensions> for (u32, u32) {
	fn from(dimensions: &ImageDimensions) -> (u32, u32) {
		(dimensions.width, dimensions.height)
 	}
}