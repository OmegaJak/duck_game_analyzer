use std::{error::Error, fs, path::Path};

use chrono::NaiveDateTime;

pub fn get_album_datetimes(folder_path: &str) -> Result<Vec<NaiveDateTime>, Box<dyn Error>> {
    let album_folder = Path::new(folder_path);
    
    let entries = fs::read_dir(album_folder)?;
    let entries = entries.map(|e| e.unwrap()).collect::<Vec<_>>();
    println!("Found {} files", entries.len());

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