mod music;
extern crate std;
#[macro_use]
extern crate rand_derive;

use anyhow::{Context, Result};
use dialoguer::MultiSelect;
use getopts::Options;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::LineWriter;
use std::io::Read;
use std::io::Write;
use std::io::{self, BufRead};
use std::path::Path;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[derive(Debug, Serialize, Deserialize)]
struct PratikConfig {
    data_folder: String,
    init_practice_num: i32,
    min_practice: i32,
}

fn load_config() -> Result<PratikConfig> {
    let args: Vec<String> = env::args().collect();
    let mut options = Options::new();
    options.optopt("c", "config", "config file path", "config.yml");
    let matches = match options.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!(f.to_string())
        }
    };
    let config_path = matches
        .opt_str("c")
        .context("Missing config file path!")
        .unwrap();
    let mut contents = String::new();
    let mut file = File::open(&config_path)
        .context("Config file path does not exists!")
        .unwrap();
    file.read_to_string(&mut contents)?;
    let config: PratikConfig = serde_yaml::from_str(&contents)?;
    return Ok(config);
}

fn load_data(config: &PratikConfig) -> std::result::Result<Vec<music::Practice>, anyhow::Error> {
    let path = &config.data_folder;
    let fpath = Path::new(path);
    if fpath.exists() {
        let data_ls: Vec<music::Practice> = read_lines(path)?
            .map_ok(|line: String| {
                let e: music::Practice = serde_json::from_str(&line).unwrap();
                return e;
            })
            .map(Result::unwrap)
            .collect_vec();
        return Ok(data_ls);
    } else {
        println!("No data found @ {}", path);
        let new_data = music::Practice::init_dataset(config.init_practice_num)?;
        println!("Init new dataset @ {}", path);
        let file = File::create(&fpath).unwrap();
        let mut file_writer = LineWriter::new(file);
        new_data.iter().for_each(|x| {
            file_writer
                .write_all(x.to_json().unwrap().as_bytes())
                .unwrap();
            file_writer.write_all(b"\n").unwrap();
        });
        return Ok(new_data);
    };
}

fn dump_data(config: &PratikConfig, dataset: &Vec<music::Practice>) -> Result<()> {
    let fpath = Path::new(&config.data_folder);
    let file = File::create(&fpath).unwrap();
    let mut file_writer = LineWriter::new(file);
    dataset.iter().for_each(|x| {
        file_writer
            .write_all(x.to_json().unwrap().as_bytes())
            .unwrap();
        file_writer.write_all(b"\n").unwrap();
    });
    Ok(())
}

fn main() -> Result<()> {
    println!("Welcome back!");
    let config = load_config().unwrap();

    // load practices, if first time -> create data file
    let mut dataset = load_data(&config).unwrap();

    // show practices due today to choose from (if any)
    let needs_train = dataset.iter().filter(|x| x.needs_training()).collect_vec();
    let chosen: Vec<usize> = MultiSelect::new().items(&needs_train).interact()?;

    println!("Your practice for today is...");

    if chosen.len() < config.min_practice as usize {
        println!(
            "Less than {} practices chosen. New practices were generated!",
            config.min_practice
        );
        // Generate new practices
        let diff = (config.min_practice as usize) - chosen.len();
        for _ in 1..=diff {
            let new_pratik = music::Practice::new();
            println!("{:?}", &new_pratik.to_string());
            dataset.push(new_pratik);
        }
    }

    // Update chosen practices
    chosen.into_iter().for_each(|idx| {
        dataset[idx].update_practice();
        println!("{:?}", &dataset[idx].to_string());
    });

    // Dump dataset
    dump_data(&config, &dataset)?;

    // Prints
    Ok(())
}
