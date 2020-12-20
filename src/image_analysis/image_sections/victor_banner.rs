use std::collections::HashMap;
use lodepng::RGB;
use imgref::ImgRef;

use super::full_podium_image::FullPodiumImage;

pub struct VictorBanner<'a> {
    pub image: ImgRef<'a, RGB<u8>>
}

impl<'a> VictorBanner<'a> {
    pub const HEIGHT: usize = 23;
    pub const WIDTH: usize = 179;

    pub fn from(podium_image: &'a FullPodiumImage) -> Self {
        let (top_left_x, top_left_y) = FullPodiumImage::get_victor_banner_top_left_position();
        VictorBanner { image: podium_image.image.sub_image(top_left_x, top_left_y, VictorBanner::WIDTH, VictorBanner::HEIGHT) }
    }

    pub fn determine_white_color(&self) -> RGB<u8> {
        let pixel_counts = get_border_pixel_color_count(self.image);
        let (most_prominent_color, _) = pixel_counts.iter().max_by_key(|(_, &asdf)| asdf).unwrap();

        most_prominent_color.to_owned()
    }

    pub fn determine_black_color(&self) -> RGB<u8> {
        get_darkest_color(self.image)
    }
}

fn get_border_pixel_color_count(image: ImgRef<RGB<u8>>) -> HashMap<RGB<u8>, usize> {
    let mut border_pixels = Vec::new();
    for &row_index in [0, image.height() - 1].iter() {
        for top_row_pixel in &image[row_index] {
            border_pixels.push(top_row_pixel);
        }
    }

    for &x in [0, image.width() - 1].iter() {
        for y in 1..image.height() {
            let pixel = &image[(x, y)];
            border_pixels.push(pixel);
        }
    }

    println!("Border count: {}", border_pixels.len());
    get_color_counts(border_pixels)
}

fn get_darkest_color(image: ImgRef<RGB<u8>>) -> RGB<u8> {
    image.pixels().min_by_key(|px| px.r as usize + px.g as usize + px.b as usize).unwrap()
}

fn get_color_counts<'a, I>(pixels: I) -> HashMap<RGB<u8>, usize>
    where I: IntoIterator<Item = &'a RGB<u8>>
{
    let mut all_colors = HashMap::new();
    for &pixel in pixels {
        *all_colors.entry(pixel).or_insert(0) += 1;
    }

    all_colors
}

#[cfg(test)]
mod tests {
    use imgref::Img;
	use super::*;
	use crate::{constants::*, test_helpers::*};

    #[test]
    fn can_get_border_pixel_color_count() {
        let mut image = Img::new(vec![BLACK; 1000], 100, 10);
        let count_map = get_border_pixel_color_count(image.as_ref());
        assert_eq!(1, count_map.len());

        let different_black_instance = RGB { r: 0, g: 0, b: 0 };
        assert_eq!(true, count_map.contains_key(&different_black_instance));
        assert_eq!(218, *count_map.get(&BLACK).unwrap());

        image[(99_usize, 0_usize)] = WHITE;
        image[(0_usize, 5_usize)] = RGB { r: 255, g: 255, b: 255 };

        let count_map = get_border_pixel_color_count(image.as_ref());
        assert_eq!(2, count_map.len());
        assert_eq!(216, *count_map.get(&BLACK).unwrap());
        assert_eq!(2, *count_map.get(&WHITE).unwrap());
    }

    #[test]
    fn can_determine_victor_banner_white_color() {
        assert_expected_white_color("05-03-19 23;23", RGB { r: 252, g: 198, b: 162 });
        assert_expected_white_color("05-27-18 18;03", RGB { r: 232, g: 232, b: 232 });
        assert_expected_white_color("09-09-18 1;11", RGB { r: 207, g: 206, b: 247 });
    }

    #[test]
    fn can_determine_victor_banner_black_color() {
        assert_expected_black_color("04-08-18 20;09", RGB { r: 0, g: 0, b: 3 });
        assert_expected_black_color("04-09-17 0;18", RGB { r: 10, g: 3, b: 17 });
        assert_expected_black_color("04-12-17 23;58", BLACK);
        assert_expected_black_color("04-12-19 22;29", RGB { r: 0, g: 0, b: 6 });
    }

    fn assert_expected_white_color(filename: &str, expected_white: RGB<u8>) {
        let image = get_image(filename);
        let victor_banner = VictorBanner::from(&image);

        let actual_white = victor_banner.determine_white_color();
        assert_eq!(expected_white, actual_white);
    }

    fn assert_expected_black_color(filename: &str, expected_black: RGB<u8>) {
        let image = get_image(filename);
        let victor_banner = VictorBanner::from(&image);

        let actual_black = victor_banner.determine_white_color();
        assert_eq!(expected_black, actual_black);
    }
}