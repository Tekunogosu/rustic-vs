mod mod_api_struct;

use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};
use colored::Colorize;
use zip;
use json5;
use prettytable::{format, Cell, Row, Table};
use reqwest::StatusCode;
use serde_json::Value;
use tokio;
use crate::mod_api_struct::{Mod, ModJson};

use rayon::prelude::*;

// #[allow(dead_code)]
// fn list_files(dir_path: &str) -> Result<(), Box<dyn Error>> {
//     let entries = fs::read_dir(dir_path)?;
//
//     let mut entries_vec: Vec<_> = entries.filter_map(|e| e.ok()).collect();
//
//     entries_vec.sort_by(|a, b| {
//         let a_name = a.file_name().to_string_lossy().to_lowercase();
//         let b_name = b.file_name().to_string_lossy().to_lowercase();
//         a_name.cmp(&b_name)
//     });
//
//     for entry in entries_vec {
//         let path = entry.path();
//
//         // get the file name
//         let file_name = path.file_name().unwrap().to_string_lossy();
//
//         let file_type = if path.is_dir() { "directory" } else { "file" };
//
//         // try te get the file size
//         let size = if path.is_file() {
//             match fs::metadata(&path) {
//                 Ok(metadata) => metadata.len().to_string(),
//                 Err(_) => "unknown".to_string(),
//             }
//         } else {
//             "-".to_string()
//         };
//
//         eprintln!("{:<50} {:<10} {:10} bytes", file_name.blue(), file_type.yellow(), size.blue());
//     }
//
//     Ok(())
// }


fn validate_dir(dir: &str) -> Result<(), String> {
    let path = Path::new(dir);
    if !path.exists() {
        return Err(format!("{} does not exist", dir));
    }

    if !path.is_dir() {
        return Err(format!("{} is not a directory", dir));
    }

    match fs::read_dir(path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Can't access '{}': {}", dir, e)),
    }
}

fn get_case_insensitive<'a>(obj: &'a Value, key: &str) -> Option<&'a Value> {
    if let Some(obj) = obj.as_object() {
        obj.iter()
            .find(|(k, _)|k.to_lowercase() == key.to_lowercase())
            .map(|(_, v)| v)
    } else {
        None
    }
}

async fn fetch_mod_from_id(mod_id: &str) -> Result<ModJson, Box<dyn Error>> {
    let url = format!("https://mods.vintagestory.at/api/mod/{}", &mod_id.trim_matches('"'));
    println!("Fetching {}", url);
    let result = reqwest::get(&url).await?;

    match result.status() {
        StatusCode::OK => {
            let text = result.json::<Mod>().await?;
            Ok(text.mod_json)
        },
        _ => {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Can't fetch modid: {}", mod_id.red()),
            )))

        }
    }
}

struct TableData {
    name: String,
    modid: String,
    author: String,
    installed_version: String,
    latest_version: String,
    is_updated: bool,
}

async fn process_zip(file_path: &Path) -> Result<Vec<TableData>, Box<dyn Error>> {

    let file = File::open(file_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut out_data: Vec<TableData> = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        // This can be shortened, but is here to show how it can be done
        // json5 uses the serde deserialization system
        let contents : Option<serde_json::Value> = if name == "modinfo.json" {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            Some(json5::from_str(&contents)?)
        } else {
            None
        };

        if let Some(c) = contents {
            // eprintln!("{}", format!("modinfo.json: {}", c).green());

            let mod_name = get_case_insensitive(&c, "name").unwrap().to_string();
            let mod_id = get_case_insensitive(&c, "modid").unwrap().to_string();
            let mod_author = get_case_insensitive(&c, "authors").unwrap().as_array().map_or(String::new(), |arr|
                arr.iter().filter_map(|v| v.as_str()).collect::<Vec<&str>>().join(",")
            );
            let mod_version = get_case_insensitive(&c, "version").unwrap().to_string();

            // send the http reqwests for the mod
            let mod_result = fetch_mod_from_id(&mod_id).await?;

            let latest_version = mod_result.releases[0].modversion.to_owned().unwrap();


            out_data.push(TableData {
                name: mod_name,
                modid: mod_id,
                author: mod_author,
                installed_version: mod_version.clone(),
                latest_version: latest_version.clone(),
                is_updated: latest_version == mod_version.trim_matches('"'),

            })
        }
    }

    Ok(out_data)
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // get the directory path from the command line
    // lif no argument is provided, list the current directory

    // get the path to mods, ~/.config/VintagestoryData/Mods by default
    // iterate through list of mods
    // foreach mod, check

    let mut user_home = dirs::home_dir().unwrap();

    // todo: check for windows and handle that accordingly
    user_home.push(".config/VintagestoryData/Mods");

    let mut dir_to_scan = user_home.to_string_lossy().to_string();

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        dir_to_scan = args[1].trim().to_string();
        eprintln!("Scanning: {}", dir_to_scan.blue().bold());
    } else {
        eprintln!("Scanning default path of: {}", dir_to_scan.blue().bold());
    }

    // check if the directory is valid, then do everything else
    if let Err(error) = validate_dir(&dir_to_scan) {
        eprintln!("{}", format!("{}", error).red().bold());
        std::process::exit(1);
    }

    // list_files(dir_to_scan.as_str())
    let entries: Vec<_> = fs::read_dir(dir_to_scan).unwrap().collect();

    let mut table = Table::new();

    table.set_format(*format::consts::FORMAT_DEFAULT);
    table.set_titles(Row::new(vec![
        Cell::new("Mod Name").style_spec("Fb+"),
        Cell::new("ModID").style_spec("Fb+"),
        Cell::new("Author(s)").style_spec("Fb+"),
        Cell::new("Installed Version").style_spec("Fb+"),
        Cell::new("Latest Version").style_spec("Fb+"),
        Cell::new("Is Updated").style_spec("Fb+"),
    ]));

    let results = Arc::new(Mutex::new(Vec::new()));
    let thread_results = Arc::clone(&results);

    entries.into_par_iter()
        .filter_map(Result::ok)
        .for_each(|entry| {
            let path = entry.path();
            // check only the zip files
            if path.is_file() && path.extension().map_or(false, |ext| ext == "zip") {
                eprintln!("Checking file: {}", path.to_string_lossy().blue().bold());
                let rt = tokio::runtime::Runtime::new().unwrap();
                let mod_info = rt.block_on(process_zip(&path)).unwrap();

                for i in mod_info {
                    let is_updated = if i.is_updated {
                        Cell::new("True").style_spec("Fg") } else { Cell::new("False").style_spec("FR") };

                    let row = Row::new(vec![
                        Cell::new(i.name.as_str()).style_spec("FC"),
                        Cell::new(i.modid.as_str()).style_spec("FC"),
                        Cell::new(i.author.as_str()).style_spec("FC"),
                        Cell::new(i.installed_version.as_str().trim_matches('"')).style_spec("Fw"),
                        Cell::new(i.latest_version.as_str()).style_spec(format!("F{}", if i.is_updated { "g" } else { "r" }).as_str()),
                        is_updated,]);

                    thread_results.lock().unwrap().push(row);
                }
            }
    });

    for row in results.lock().unwrap().iter() {
        table.add_row(row.clone());
    }

    table.printstd();

    Ok(())
}
