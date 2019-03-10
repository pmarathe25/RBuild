use std::env;
use std::vec::Vec;
use std::string::String;
use std::collections::HashSet;

mod graph;
use crate::graph::Graph;

mod threadpool;
use crate::threadpool::{ThreadPool, WorkerStatus};

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

    // First argument is the executable name and the second is the rbuild file.
    // The remaining arguments are targets.
    let target_indices = args.iter().skip(2).map(
        |target_path| match graph.get_index(&target_path) {
            Some(index) => *index,
            None => panic!("{} is not a valid target", target_path),
        });


    // DEBUG:
    // let (deps, depless) = graph.get_deps(target_indices);
    // println!("Dependencies for {:?}: {:?}\nNodes with no dependencies: {:?}", args.iter().skip(2).collect::<HashSet<_>>(), deps.iter().map(|x| graph.nodes.get(*x).unwrap().path).collect::<HashSet<_>>(), depless.iter().map(|x| graph.nodes.get(*x).unwrap().path).collect::<HashSet<_>>());

    // Threadpool stuff
    let pool = ThreadPool::new(8);

    let num_jobs = 8;
    for i in 0..num_jobs {
        pool.execute(move || std::thread::sleep(std::time::Duration::new(i, 0)));
    }

    let mut jobs_left = num_jobs;
    while jobs_left > 0 {
        match pool.wstatus_receiver.recv().unwrap() {
            WorkerStatus::Complete(id) => println!("{} finished!", id),
        };
        jobs_left -= 1;
    }

    // std::thread::sleep(std::time::Duration::new(3, 0));
}
