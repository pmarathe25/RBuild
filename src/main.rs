use std::env;
use std::vec::Vec;
use std::io::Write;
use std::process::Command;
use std::string::String;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

mod graph;
use crate::graph::Graph;

mod threadpool;
use crate::threadpool::{ThreadPool, WorkerStatus, ExecNode};

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

    // let mut graph = Graph::from_config(&config);
    // // DEBUG:
    // println!("Graph:\n{}", graph);
    //
    // // First argument is the executable name and the second is the rbuild file.
    // // The remaining arguments are targets.
    // let target_indices = args.iter().skip(2).map(
    //     |target_path| match graph.get_index(&target_path) {
    //         Some(index) => *index,
    //         None => panic!("{} is not a valid target", target_path),
    //     });
    //
    // let (deps, depless) = graph.get_deps(target_indices);
    // println!("Dependencies for {:?}: {:?}\nNodes with no dependencies: {:?}", args.iter().skip(2).collect::<HashSet<_>>(), deps.iter().map(|x| graph.nodes.get(*x).unwrap().path).collect::<HashSet<_>>(), depless.iter().map(|x| graph.nodes.get(*x).unwrap().path).collect::<HashSet<_>>());

    // Threadpool stuff

    // let num_jobs = 12;
    // for i in 0..num_jobs {
    //     pool.execute(move || std::thread::sleep(std::time::Duration::new(i as u64, 0)), i);
    // }

    // println!("All jobs queued!");
    // let mut jobs_left = num_jobs;
    // while jobs_left > 0 {
    //     match pool.wstatus_receiver.recv().unwrap() {
    //         WorkerStatus::Complete(id, job_id) => println!("{} finished {}!", id, job_id),
    //     };
    //     jobs_left -= 1;
    // }
    let n = Arc::new(Mutex::new(ExecNode::new()));
    println!("{:?}", n);
     {
         let pool = ThreadPool::new(8);

         pool.execute(&n, 0);


         // for node in graph.nodes.iter() {
         //     let cloned_cmds = Arc::clone(&node.cmds);
         //     pool.execute(move || {
         //        for cmd in cloned_cmds.lock().unwrap().iter_mut() {
         //            cmd.spawn();
         //        }
         //     }, 0);
             // for cmd in node.cmds.iter() {
             //     pool.execute(move || {cmd_cloned.execute();}, 0)
             // }
         // }
     }

     println!("{:?}", n);

    // for i in 0..8 {
    //     pool.execute(move || {Command::new("echo").arg("Hi!").spawn().unwrap();}, i)
    // }
}
