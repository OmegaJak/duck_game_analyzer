use lodepng::RGB;
use imgref::ImgRef;

use crate::constants::WHITE;

pub struct ScorePlacard<'a> {
    image: ImgRef<'a, RGB<u8>>
}

impl ScorePlacard<'_> {
    pub fn is_score_placard(maybe_placard: ImgRef<RGB<u8>>) -> bool {
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