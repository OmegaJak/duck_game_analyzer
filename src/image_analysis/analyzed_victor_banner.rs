use imgref::{Img, ImgRef, ImgVec};
use lodepng::RGB;

use super::image_sections::victor_banner::VictorBanner;

pub struct AnalyzedVictorBanner {
    image: ImgVec<AnalyzedBannerPixel>
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum AnalyzedBannerPixel {
    Invalid,
    White,
    Black
}



fn coordinates<T>(image: ImgRef<T>) -> impl Iterator<Item = (usize, usize)> {
    let ys = 0..image.height();
    let width = image.width();

    ys.flat_map(move |y| std::iter::repeat(y).zip(0..width)).map(|(y, x)| (x, y))
}

impl AnalyzedVictorBanner {
    pub fn from(victor_banner: &VictorBanner) -> Self {
        let width = victor_banner.image.width();
        let height = victor_banner.image.height();
        let banner_white = victor_banner.determine_white_color();
        let banner_black = victor_banner.determine_black_color();

        let analyzed_pixels: Vec<AnalyzedBannerPixel> = vec![AnalyzedBannerPixel::Invalid; width * height];
        let mut analyzed_image = Img::new(analyzed_pixels, width, height);
        for (x, y) in coordinates(victor_banner.image) {
            let analyzed_pixel = Self::analyze_pixel(victor_banner.image, x, y, banner_white, banner_black);
            analyzed_image[(x, y)] = analyzed_pixel;
        }

        AnalyzedVictorBanner { image: analyzed_image }
    }

    fn analyze_pixel(image: ImgRef<RGB<u8>>, x: usize, y: usize, banner_white: RGB<u8>, banner_black: RGB<u8>) -> AnalyzedBannerPixel {
        let original_pixel = image[(x, y)];
        if original_pixel == banner_white && Self::is_pixel_surrounded_by_black_and_white(image, x, y, banner_white, banner_black) {
            AnalyzedBannerPixel::White
        } else if original_pixel == banner_black && Self::is_pixel_surrounded_by_black_and_white(image, x, y, banner_white, banner_black)  {
            AnalyzedBannerPixel::Black
        } else {
            AnalyzedBannerPixel::Invalid
        }
    }

    fn is_pixel_surrounded_by_black_and_white(image: ImgRef<RGB<u8>>, pixel_x: usize, pixel_y: usize, white: RGB<u8>, black: RGB<u8>) -> bool{
        let pixel_x = pixel_x as isize;
        let pixel_y = pixel_y as isize;

        Self::is_pixel_black_or_white(image, pixel_x - 1, pixel_y, white, black)
        && Self::is_pixel_black_or_white(image, pixel_x + 1, pixel_y, white, black)
        && Self::is_pixel_black_or_white(image, pixel_x, pixel_y - 1, white, black)
        && Self::is_pixel_black_or_white(image, pixel_x, pixel_y + 1, white, black)
    }

    fn is_pixel_black_or_white(image: ImgRef<RGB<u8>>, x: isize, y: isize, white: RGB<u8>, black: RGB<u8>) -> bool {
        if x < 0 || x > (image.width() as isize) - 1 || y < 0 || y > (image.height() as isize) - 1 {
            return true;
        }

        let pixel = image[(x as usize, y as usize)];
        pixel == white || pixel == black
    }

    fn matches(&self, other: &AnalyzedVictorBanner) -> bool {
        for coordinate in coordinates(self.image.as_ref()) {
            match (self.image[coordinate], other.image[coordinate]) {
                (AnalyzedBannerPixel::Invalid, _) | (_, AnalyzedBannerPixel::Invalid) => { }
                (self_pixel, other_pixel) if self_pixel != other_pixel => { return false }
                _ => { }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{constants::*, test_helpers::*};
    use crate::iter_ext::IterExt;
    use ntest::*;

    #[test]
    fn can_analyze_pixel() {
        let image = get_image("04-25-20 21;01");
        let banner_white = RGB { r: 232, g: 232, b: 232 };
        let banner_black = BLACK;

        let analyze_at = |x: usize, y: usize| AnalyzedVictorBanner::analyze_pixel(image.image.as_ref(), x, y, banner_white, banner_black);

        assert_eq!(AnalyzedBannerPixel::Invalid, analyze_at(141, 47));
        assert_eq!(AnalyzedBannerPixel::Invalid, analyze_at(140, 47));
        assert_eq!(AnalyzedBannerPixel::Black, analyze_at(139, 47));
        assert_eq!(AnalyzedBannerPixel::White, analyze_at(137, 42));
        assert_eq!(AnalyzedBannerPixel::Invalid, analyze_at(138, 39));
        assert_eq!(AnalyzedBannerPixel::Invalid, analyze_at(138, 35));
        // assert_eq!(AnalyzedBannerPixel::Invalid, analyze_at(139, 35)); //?????
    }

    #[test]
    fn can_analyze_victor_banner() {
        let image = get_image("02-10-17 16;18");
        let victor_banner = VictorBanner::from(&image);
        let analyzed_victor_banner = AnalyzedVictorBanner::from(&victor_banner);

        assert_eq!(victor_banner.image.width(), analyzed_victor_banner.image.width());
        assert_eq!(victor_banner.image.height(), analyzed_victor_banner.image.height());

        let invalid_count = analyzed_victor_banner.image.pixels().filter_count(|&p| p == AnalyzedBannerPixel::Invalid);
        assert_eq!(0, invalid_count);
    }

    #[test]
    fn banner_matches_itself() {
        let victor_banner = get_analyzed_victor_banner("11-08-19 20;08");
        assert_true!(victor_banner.matches(&victor_banner));
    }

    #[test]
    fn clear_banner_matches_partially_obscured_banner() {
        let clear_victor_banner = get_analyzed_victor_banner("11-16-19 14;43");
        let partially_obscured_victor_banner = get_analyzed_victor_banner("11-09-19 19;39");

        assert_true!(clear_victor_banner.matches(&partially_obscured_victor_banner));
    }

    #[test]
    fn completely_different_banners_do_not_match() {
        let omegajak_banner = get_analyzed_victor_banner("11-16-19 14;43");
        let tewny_banner = get_analyzed_victor_banner("11-08-19 19;40");

        assert_false!(omegajak_banner.matches(&tewny_banner));
    }

    fn get_analyzed_victor_banner(filename_date: &str) -> AnalyzedVictorBanner {
        let image = get_image("02-10-17 16;18");
        let victor_banner = VictorBanner::from(&image);
        AnalyzedVictorBanner::from(&victor_banner)
    }
}