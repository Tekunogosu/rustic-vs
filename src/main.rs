
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use colored::Colorize;
use zip;
use json5;
use prettytable::{cell, format, row, Cell, Row, Table};
use reqwest::Response;
use serde_json::Value;
use tokio;

#[allow(dead_code)]
fn list_files(dir_path: &str) -> Result<(), Box<dyn Error>> {
    let entries = fs::read_dir(dir_path)?;

    let mut entries_vec: Vec<_> = entries.filter_map(|e| e.ok()).collect();

    entries_vec.sort_by(|a, b| {
        let a_name = a.file_name().to_string_lossy().to_lowercase();
        let b_name = b.file_name().to_string_lossy().to_lowercase();
        a_name.cmp(&b_name)
    });

    for entry in entries_vec {
        let path = entry.path();

        // get the file name
        let file_name = path.file_name().unwrap().to_string_lossy();

        let file_type = if path.is_dir() { "directory" } else { "file" };

        // try te get the file size
        let size = if path.is_file() {
            match fs::metadata(&path) {
                Ok(metadata) => metadata.len().to_string(),
                Err(_) => "unknown".to_string(),
            }
        } else {
            "-".to_string()
        };

        eprintln!("{:<50} {:<10} {:10} bytes", file_name.blue(), file_type.yellow(), size.blue());
    }

    Ok(())
}


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

async fn fetch_mod_from_id(mod_id: &str) -> Result<String, Box<dyn Error>> {
    let url = format!("https://mods.vintagestory.at/api/mod/{}", &mod_id.trim_matches('"'));
    println!("Fetching {}", url);
    Ok(reqwest::Client::new().get(url).send().await?.text().await?)
}

// fn check_code(code: u16) -> String {
//
// }

async fn check_version(blob: &str) -> Result<(), Box<dyn Error>> {
    // get json blob from http request
    // compare version against the current version
    // call update_mod()

    // get the version from the blob
    // mod -> releases[0] -> modversion

    let json = json5::from_str::<Value>(&blob)?;
    // let version = json.get("mod").unwrap().as_str().unwrap()

    match json {
        Ok(T) => {
            eprintln!("Checking {}", blob);
        },
        None => {
            println!("{} is not a JSON object", blob);
        }
    }


    Ok(())
}

async fn process_zip(file_path: &Path) -> Result<Vec<String>, Box<dyn Error>> {

    let file = File::open(file_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut out_vec = Vec::new();

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
            // get
            eprintln!("{}", format!("modinfo.json: {}", c).green());

            let mod_name = get_case_insensitive(&c, "name").unwrap().to_string();
            let mod_id = get_case_insensitive(&c, "modid").unwrap().to_string();
            let mod_author = get_case_insensitive(&c, "authors").unwrap().as_array().map_or(String::new(), |arr|
                arr.iter().filter_map(|v| v.as_str()).collect::<Vec<&str>>().join(",")
            );
            let mod_version = get_case_insensitive(&c, "version").unwrap().to_string();

            // send the http reqwests for the mod
            match fetch_mod_from_id(&mod_id).await {
                Ok(data) => {
                    check_version(&data).await?;
                }
                _ => {}
            }

            out_vec.push(mod_name);
            out_vec.push(mod_id);
            out_vec.push(mod_author);
            out_vec.push(mod_version);
        }
    }

    Ok(out_vec)
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
    let entries = fs::read_dir(dir_to_scan)?;

    let mut table = Table::new();

    table.set_format(*format::consts::FORMAT_DEFAULT);
    table.set_titles(Row::new(vec![
        Cell::new("Mod Name").style_spec("FbBd+"),
        Cell::new("ModID").style_spec("FbBd+"),
        Cell::new("Author(s)").style_spec("FbBd+"),
        Cell::new("Version").style_spec("FbBd+"),
    ]));

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        // check only the zip files
        if path.is_file() && path.extension().map_or(false, |ext| ext == "zip") {
            eprintln!("Checking file: {}", path.to_string_lossy().blue().bold());
            let mod_info = process_zip(&path).await?;
            table.add_row(Row::new(mod_info.iter().map(|v| Cell::new(v)).collect()));
        }
    }

    table.printstd();

    Ok(())
}
