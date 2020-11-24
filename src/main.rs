use std::{path::Path, fs, io};

use chrono::{DateTime, NaiveDateTime};

fn main() {
    print_directory().expect("!?!?!?!?");
}

fn print_directory() -> io::Result<()> {
    let album_folder = Path::new(&r"C:\Users\JAK\Documents\DuckGame\Album");
    
    if album_folder.is_dir() {
        let entries = fs::read_dir(album_folder)?;
        let entries = entries.map(|e| e.unwrap()).collect::<Vec<_>>();
        println!("Found {} files", entries.len());

        let filenames = entries.iter().filter_map(|e| get_filename_without_extension(e)).collect::<Vec<String>>();
        //println!("{:#?}", filenames);

        // let mut filenames = Vec::new();
        // for entry in entries {
        //     // let filename = get_filename_without_extension(&entry);
        //     // if let Some(name) = filename {
        //     //     filenames.push(name);
        //     // }

        //     let filename = entry.file_name();
        //     let filename = filename.to_str();
        //     if let Some(name) = filename {
        //         let filename = name.trim_end_matches(".png");
        //         filenames.push(String::from(filename))
        //     }
        // }

        let mut datetimes = filenames.iter().map(|f| NaiveDateTime::parse_from_str(f, "%m-%d-%y %H;%M").expect("Failed to parse datetime")).collect::<Vec<_>>();
        datetimes.sort();
        println!("{:#?}", datetimes);
    } else {
        println!("Given path is not a directory!");
    }

    Ok(())
}

fn get_filename_without_extension(dir_entry: &fs::DirEntry) -> Option<String> {
    let filename = dir_entry.file_name();
    let filename = filename.to_str()?;
    let filename = filename.trim_end_matches(".png");

    Some(String::from(filename))
}