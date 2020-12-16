#[path = "iter_ext.rs"]
mod iter_ext;

use std::{error::Error, collections::HashMap, fs, path::Path};
use chrono::NaiveDateTime;
use fs::DirEntry;
use imgref::{Img, ImgRef, ImgVec};
use iter_ext::IterExt;
use lodepng::{Image, RGB};

const WHITE: RGB<u8> = RGB { r: 255, g: 255, b: 255 };
const BLACK: RGB<u8> = RGB { r: 0, g: 0, b: 0 };

pub fn get_album_files(folder_path: &str) -> Result<Vec<DirEntry>, Box<dyn Error>> {
    let album_folder = Path::new(folder_path);
    
    let entries = fs::read_dir(album_folder)?;
    let entries = entries.map(|e| e.unwrap()).collect::<Vec<_>>();
    println!("Found {} files", entries.len());

    Ok(entries)
}

pub fn get_album_datetimes(folder_path: &str) -> Result<Vec<NaiveDateTime>, Box<dyn Error>> {
    let entries = get_album_files(folder_path)?;

    let filenames = entries.iter().filter_map(|e| get_filename_without_extension(e)).collect::<Vec<String>>();
    let mut datetimes = filenames.iter().map(|f| NaiveDateTime::parse_from_str(f, "%m-%d-%y %H;%M").expect("Failed to parse datetime")).collect::<Vec<_>>();
    datetimes.sort();

    Ok(datetimes)
}

fn get_filename_without_extension(dir_entry: &fs::DirEntry) -> Option<String> {
    let filename = dir_entry.file_name();
    let filename = filename.to_str()?;
    let filename = filename.trim_end_matches(".png");

    Some(String::from(filename))
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum AnalyzedBannerPixel {
    Invalid,
    White,
    Black
}

struct FullPodiumImage {
    filepath: String,
    image: ImgVec<RGB<u8>>
}

struct ScorePlacard<'a> {
    image: ImgRef<'a, RGB<u8>>
}

struct VictorBanner<'a> {
    image: ImgRef<'a, RGB<u8>>
}

struct AnalyzedVictorBanner {
    image: ImgVec<AnalyzedBannerPixel>
}

impl FullPodiumImage {
    pub fn at_path<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let image = lodepng::decode_file(path.as_ref(), lodepng::ColorType::RGB, 8)?;
        if let Image::RGB(image) = image {
            let image = Img::new(image.buffer, image.width, image.height);
            let filepath = String::from(path.as_ref().to_str().expect("Failed to get string for path"));
            Ok(FullPodiumImage { image, filepath })
        } else {
            Err("Failed to match image to RGB image")?
        }
    }

    pub fn get_player_count(&self) -> u32 {
        let four_players_placard_positions = [(85, 149), (127, 149), (169, 149), (211, 149)];
        let three_players_placard_positions = [(106, 149), (148, 149), (190, 149)];
        
        let four_player_positions_are_placards = self.are_all_top_lefts_placards(&four_players_placard_positions);
        if four_player_positions_are_placards.iter().all(|b| *b) {
            4
        } else if four_player_positions_are_placards[1] && four_player_positions_are_placards[2] {
            2
        } else {
            let three_player_positions_are_placards = self.are_all_top_lefts_placards(&three_players_placard_positions);
            if three_player_positions_are_placards.iter().all(|b| *b) {
                3
            } else {
                panic!("Couldn't determine number of players!");
            }
        }
    }

    fn are_all_top_lefts_placards(&self, coords: &[(usize, usize)]) -> Vec<bool> {
        coords.iter().map(|pos| self.is_top_left_of_score_placard_at(*pos)).collect::<Vec<bool>>()
    }
    
    fn is_top_left_of_score_placard_at(&self, coord: (usize, usize)) -> bool {
        let placard_width = 21;
        let placard_height = 8;
        let (left, top) = coord;
        ScorePlacard::is_score_placard(self.image.sub_image(left, top, placard_width, placard_height))
    }

    pub fn get_victor_banner_top_left_position() -> (usize, usize) {
        (72, 35)
    }
}

impl ScorePlacard<'_> {
    fn is_score_placard(maybe_placard: ImgRef<RGB<u8>>) -> bool {
        let mut unique_colors: Vec<RGB<u8>> = Vec::new();
        for pixel in maybe_placard.pixels() {
            if !unique_colors.contains(&pixel) {
                unique_colors.push(pixel);
            }
        }
    
        //println!("{:#?}", unique_colors);
        unique_colors.len() == 2 || (unique_colors.len() == 3 && unique_colors.iter().any(|pix| pix == &WHITE))
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

impl<'a> VictorBanner<'a> {
    pub const HEIGHT: usize = 23;
    pub const WIDTH: usize = 179;

    pub fn from(podium_image: &'a FullPodiumImage) -> Self {
        let (top_left_x, top_left_y) = FullPodiumImage::get_victor_banner_top_left_position();
        VictorBanner { image: podium_image.image.sub_image(top_left_x, top_left_y, VictorBanner::WIDTH, VictorBanner::HEIGHT) }
    }

    fn determine_white_color(&self) -> RGB<u8> {
        let pixel_counts = get_border_pixel_color_count(self.image);
        let (most_prominent_color, _) = pixel_counts.iter().max_by_key(|(_, &asdf)| asdf).unwrap();

        most_prominent_color.to_owned()
    }

    fn determine_black_color(&self) -> RGB<u8> {
        get_darkest_color(self.image)
    }
}

impl AnalyzedVictorBanner {
    pub fn from(victor_banner: &VictorBanner) -> Self {
        let width = victor_banner.image.width();
        let height = victor_banner.image.height();
        let banner_white = victor_banner.determine_white_color();
        let banner_black = victor_banner.determine_black_color();

        let analyzed_pixels: Vec<AnalyzedBannerPixel> = vec![AnalyzedBannerPixel::Invalid; width * height];
        let mut analyzed_image = Img::new(analyzed_pixels, width, height);
        for x in 0..width {
            for y in 0..height {
                let analyzed_pixel = Self::analyze_pixel(victor_banner.image, x, y, banner_white, banner_black);
                analyzed_image[(x, y)] = analyzed_pixel;
            }
        }

        AnalyzedVictorBanner { image: analyzed_image }
    }

    pub fn analyze_pixel(image: ImgRef<RGB<u8>>, x: usize, y: usize, banner_white: RGB<u8>, banner_black: RGB<u8>) -> AnalyzedBannerPixel {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_load_image() {
        let podium_image = get_image("12-15-16 18;03");
        let test_pixel = podium_image.image[(20_u32, 61_u32)];
        let expected_test_pixel = RGB { r: 184_u8, g: 106_u8, b: 0_u8 };
        assert_eq!(expected_test_pixel, test_pixel);
    }

    #[test]
    fn can_determine_if_placard_is_at_pixel() {
        let podium_image = get_image("12-15-16 18;50");
        let verify_are_placards = [(85, 149), (127, 149), (169, 149), (211, 149)];
        let verify_are_not_placards = [(19, 143), (250, 56), (153, 149)];

        for &expected_is_placard in verify_are_placards.iter() {
            assert_eq!(true, podium_image.is_top_left_of_score_placard_at(expected_is_placard), "{:#?}", expected_is_placard);
        }

        for &expected_not_placard in verify_are_not_placards.iter() {
            assert_eq!(false, podium_image.is_top_left_of_score_placard_at(expected_not_placard), "{:#?}", expected_not_placard);
        }
    }

    #[test]
    fn can_determine_player_count() {
        let image_expected_count_pairs = [("12-15-16 18;50", 4), ("11-22-19 18;51", 3), ("10-18-16 17;45", 2), ("10-16-16 15;22", 4)];
        for (filename, expected_count) in image_expected_count_pairs.iter() {
            let podium_image = get_image(filename);
            assert_eq!(*expected_count, podium_image.get_player_count(), "{}", filename);
        }
    }

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
    
    fn get_image(filename_date: &str) -> FullPodiumImage {
        FullPodiumImage::at_path(format!("C:\\Users\\JAK\\Documents\\DuckGame\\Album\\{}.png", filename_date)).expect("Failed to load image")
    }
}