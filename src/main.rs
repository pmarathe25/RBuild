use std::env;
use std::vec::Vec;
use std::string::String;

mod graph;
use crate::graph::Graph;

fn help(executable: &str) {
    println!("Usage: {} RBUILD_FILE [TARGETS...]
    RBUILD_FILE     Path to the rbuild configuration file.
    TARGETS         Zero or more targets to execute.", executable);
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        help(&args[0]);
    }

    let config = match std::fs::read_to_string(&args[1]) {
        Err(what) => panic!("Could not open provided configuration file {}: {}", args[1], what),
        Ok(file) => file,
    };

    let graph = Graph::from_config(&config);
    // DEBUG:
    println!("Graph contains: {:?}\n", graph);

    // First argument is the executable name and the second is the rbuild file.
    println!("{} using config file {}", args[0], args[1]);
    for target in args.iter().skip(2) {
        println!("Found target: {}", target);
    }
}
