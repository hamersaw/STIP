#[macro_use]
extern crate clap;
use clap::App;

mod album;
mod image;
mod node;
mod task;

use std::error::Error;

fn main() {
    let yaml = load_yaml!("clap.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    // parse subcommands
    match matches.subcommand() {
        ("album", Some(album_matches)) =>
            album::process(&matches, &album_matches),
        ("image", Some(image_matches)) =>
            image::process(&matches, &image_matches),
        ("node", Some(node_matches)) =>
            node::process(&matches, &node_matches),
        ("task", Some(task_matches)) =>
            task::process(&matches, &task_matches),
        (cmd, _) => println!("unknown subcommand '{}'", cmd),
    }
}

fn f64_opt(value: Option<&str>)
        -> Result<Option<f64>, Box<dyn Error>> {
    match value {
        Some(value) => Ok(Some(value.parse::<f64>()?)),
        None => Ok(None),
    }
}

fn i64_opt(value: Option<&str>) -> Result<Option<i64>, Box<dyn Error>> {
    match value {
        Some(value) => Ok(Some(value.parse::<i64>()?)),
        None => Ok(None),
    }
}

fn string_opt(value: Option<&str>) -> Option<String> {
    match value {
        Some(value) => Some(value.to_string()),
        None => None,
    }
}

fn u64_opt(value: Option<&str>) -> Result<Option<u64>, Box<dyn Error>> {
    match value {
        Some(value) => Ok(Some(value.parse::<u64>()?)),
        None => Ok(None),
    }
}
