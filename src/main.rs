mod file_reader;
mod plotter;
mod test_helpers;
mod iter_ext;
mod constants;

mod image_analysis {
    pub mod analyzed_victor_banner;
    pub mod image_sections {
        pub mod full_podium_image;
        pub mod score_placard;
        pub mod victor_banner;
    }
}

fn main() {
    let datetimes = file_reader::get_album_datetimes(&r"C:\Users\JAK\Documents\DuckGame\Album").expect("Failed to get datetimes from album folder");
    plotter::plot_datetimes(&datetimes).expect("Failed to plot datetimes");
}