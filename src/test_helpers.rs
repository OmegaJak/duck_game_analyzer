use crate::image_analysis::image_sections::full_podium_image::FullPodiumImage;

pub fn get_image(filename_date: &str) -> FullPodiumImage {
	FullPodiumImage::at_path(format!("C:\\Users\\JAK\\Documents\\DuckGame\\Album\\{}.png", filename_date)).expect("Failed to load image")
}