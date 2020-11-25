mod file_reader;
mod plotter;

fn main() {
    let datetimes = file_reader::get_album_datetimes(&r"C:\Users\JAK\Documents\DuckGame\Album").expect("Failed to get datetimes from album folder");
    plotter::plot_datetimes(&datetimes).expect("Failed to plot datetimes");
}