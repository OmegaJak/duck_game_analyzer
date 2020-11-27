use std::{error::Error, fs, path::Path};
use chrono::NaiveDateTime;
use fs::DirEntry;
use imgref::{Img, ImgRef, ImgVec};
use lodepng::{Image, RGB};

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

struct FullPodiumImage {
    filepath: String,
    image: ImgVec<RGB<u8>>
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
}

struct ScorePlacard<'a> {
    image: ImgRef<'a, RGB<u8>>
}

impl ScorePlacard<'_> {
    fn is_score_placard(maybe_placard: ImgRef<RGB<u8>>) -> bool {
        let mut unique_colors: Vec<RGB<u8>> = Vec::new();
        for pixel in maybe_placard.pixels() {
            if !unique_colors.contains(&pixel) {
                unique_colors.push(pixel);
            }
        }
    
        let white = RGB { r: 255_u8, g: 255_u8, b: 255_u8 };
        //println!("{:#?}", unique_colors);
        unique_colors.len() == 2 || (unique_colors.len() == 3 && unique_colors.iter().any(|pix| pix == &white))
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
    
    fn get_image(filename_date: &str) -> FullPodiumImage {
        FullPodiumImage::at_path(format!("C:\\Users\\JAK\\Documents\\DuckGame\\Album\\{}.png", filename_date)).expect("Failed to load image")
    }
}