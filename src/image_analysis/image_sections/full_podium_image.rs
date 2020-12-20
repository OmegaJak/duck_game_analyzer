use imgref::{Img, ImgVec};
use std::error::Error;
use std::path::Path;
use lodepng::{Image, RGB};

use super::score_placard::ScorePlacard;

pub struct FullPodiumImage {
    pub image: ImgVec<RGB<u8>>,
    filepath: String,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

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
}