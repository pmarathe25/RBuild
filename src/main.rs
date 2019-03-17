use std::env;
use std::string::String;
use std::time::SystemTime;
use std::collections::HashMap;

mod builder;
use builder::build_graph;

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

    // TODO: Number of threads should be configurable. Need RgParser for this.
    let (mut graph, node_map) = build_graph(&config, 8);

    let mut fetches = Vec::new();
    for target in args.iter().skip(2) {
        fetches.push(node_map.get(target).expect(
            &format!("{} was not found in the graph", target)
        ));
    }

    let recipe = graph.compile(fetches);
    let mut inputs_map = HashMap::with_capacity(recipe.inputs.len());
    for input in &recipe.inputs {
        inputs_map.insert(input.clone(), vec![SystemTime::UNIX_EPOCH]);
    }
    graph.run(&recipe, inputs_map);
}
