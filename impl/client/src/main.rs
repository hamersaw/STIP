#[macro_use]
extern crate clap;
use clap::App;

mod cluster;
mod data;
mod task;

fn main() {
    let yaml = load_yaml!("clap.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    // parse subcommands
    match matches.subcommand() {
        ("cluster", Some(cluster_matches)) =>
            cluster::process(&matches, &cluster_matches),
        ("data", Some(data_matches)) =>
            data::process(&matches, &data_matches),
        ("task", Some(task_matches)) =>
            task::process(&matches, &task_matches),
        (cmd, _) => println!("unknown subcommand '{}'", cmd),
    }
}
