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

fn open_image<P: AsRef<Path>>(path: P) -> Result<ImgVec<RGB<u8>>, Box<dyn Error>> {
    let image = lodepng::decode_file(path, lodepng::ColorType::RGB, 8)?;
    if let Image::RGB(image) = image {
        let image = Img::new(image.buffer, image.width, image.height);
        Ok(image)
    } else {
        Err("Failed to match image to RGB image")?
    }
}

fn get_player_count(image: ImgRef<RGB<u8>>) -> u32 {
    let four_players_placard_positions = [(85, 149), (127, 149), (169, 149), (211, 149)];
    let three_players_placard_positions = [(106, 149), (148, 149), (190, 149)];
    
    let four_player_positions_are_placards = are_all_top_lefts_placards(&four_players_placard_positions, image);
    if four_player_positions_are_placards.iter().all(|b| *b) {
        4
    } else if four_player_positions_are_placards[1] && four_player_positions_are_placards[2] {
        2
    } else {
        let three_player_positions_are_placards = are_all_top_lefts_placards(&three_players_placard_positions, image);
        if three_player_positions_are_placards.iter().all(|b| *b) {
            3
        } else {
            panic!("Couldn't determine number of players!");
        }
    }
}

fn are_all_top_lefts_placards(coords: &[(usize, usize)], image: ImgRef<RGB<u8>>) -> Vec<bool> {
    coords.iter().map(|pos| is_top_left_of_score_placard_at(*pos, image)).collect::<Vec<bool>>()
}

fn is_top_left_of_score_placard_at(coord: (usize, usize), image: ImgRef<RGB<u8>>) -> bool {
    let placard_width = 21;
    let placard_height = 8;
    let (left, top) = coord;
    is_score_placard(image.sub_image(left, top, placard_width, placard_height))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_load_image() {
        let image = get_image("12-15-16 18;03");
        let test_pixel = image[(20_u32, 61_u32)];
        let expected_test_pixel = RGB { r: 184_u8, g: 106_u8, b: 0_u8 };
        assert_eq!(expected_test_pixel, test_pixel);
    }

    #[test]
    fn can_determine_if_placard_is_at_pixel() {
        let image = get_image("12-15-16 18;50");
        let verify_are_placards = [(85, 149), (127, 149), (169, 149), (211, 149)];
        let verify_are_not_placards = [(19, 143), (250, 56), (153, 149)];

        for &expected_is_placard in verify_are_placards.iter() {
            assert_eq!(true, is_top_left_of_score_placard_at(expected_is_placard, image.as_ref()), "{:#?}", expected_is_placard);
        }

        for &expected_not_placard in verify_are_not_placards.iter() {
            assert_eq!(false, is_top_left_of_score_placard_at(expected_not_placard, image.as_ref()), "{:#?}", expected_not_placard);
        }
    }

    #[test]
    fn can_determine_player_count() {
        let image_expected_count_pairs = [("12-15-16 18;50", 4), ("11-22-19 18;51", 3), ("10-18-16 17;45", 2), ("10-16-16 15;22", 4)];
        for (filename, expected_count) in image_expected_count_pairs.iter() {
            let image = get_image(filename);
            assert_eq!(*expected_count, get_player_count(image.as_ref()), "{}", filename);
        }
    }
    
    fn get_image(filename_date: &str) -> ImgVec<RGB<u8>> {
        open_image(format!("C:\\Users\\JAK\\Documents\\DuckGame\\Album\\{}.png", filename_date)).expect("Failed to load image")
    }
}