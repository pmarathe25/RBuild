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

    // First argument is the executable name and the second is the rbuild file.
    // The remaining arguments are targets.
    let target_indices = args.iter().skip(2).map(
        |target_path| match graph.get_index(&target_path) {
            Some(index) => *index,
            None => panic!("{} is not a valid target", target_path),
        });

    // DEBUG:
    // let subgraph = graph.get_subgraph(target_indices);
    // for layer in &subgraph {
    //     println!("{:?}", layer.iter().map(
    //         |x| match graph.nodes.get(*x) {
    //             Some(node) => node,
    //             None => panic!("Could not find node at {}", x)
    //         }.path).collect::<Vec<&str>>());
    // }

    println!("Subgraph for lib and test: {:?}", graph.get_subgraph(vec!(0 as usize, 2 as usize).into_iter()));
    println!("Subgraph for lib and test: {:?}", graph.get_all_deps(vec!(0 as usize, 2 as usize).into_iter()));
    // println!("Subgraph for lib: {:?}", graph.get_subgraph(vec!(0 as usize).into_iter()));
    // println!("Subgraph for test: {:?}", graph.get_subgraph(vec!(2 as usize).into_iter()));
}
