use std::time::SystemTime;
use std::collections::HashMap;
use rgparse::{Parameter};

mod target;
mod token;
mod lexer;
mod parser;

fn main() {
    let mut argparser = rgparse::Parser::new("A tool for fast incremental builds");
    argparser.add_parameter(Parameter::param("--threads", "The number of threads to use during execution").alias("-t").default(&8));
    let args = argparser.parse_args();

    // Parse the config file and build a graph.
    let config_path = match args.positional.get(0) {
        Some(val) => val,
        None => {
            argparser.help();
            panic!("Error: No configuration file provided.");
        }
    };
    let config = match std::fs::read_to_string(&config_path) {
        Ok(file) => file,
        Err(what) => panic!("Error: Could not open provided configuration file {}: {}", config_path, what),
    };
    let mut parser = parser::Parser::new(&config, args.get("--threads"));
    parser.parse();

    // If graph is empty, do nothing.
    if parser.graph.len() == 0 {
        return;
    }

    // Assemble fetches, i.e. nodes to run, based on command line arguments.
    // Here we can consume the command line arguments.
    let mut fetches = Vec::new();
    for target in args.positional.into_iter().skip(1) {
        fetches.push(
            match parser.node_map.get(&target) {
                Some(&id) => id,
                None => panic!("{} is not a valid target. Note: node_map is: {:?}", target, parser.node_map),
            }
        );
    }
    // By default, we will build all the targets in the configuration file.
    if fetches.is_empty() {
        fetches = (0..parser.graph.len()).collect();
    }

    // Compile and run the graph
    // TODO: Headers/Source files with no commands may be a huge bottleneck, need to profile.
    // If so, add a fast_execute path to paragraphs.
    let recipe = parser.graph.compile(fetches);
    let mut inputs_map = HashMap::with_capacity(recipe.inputs.len());
    for input in &recipe.inputs {
        inputs_map.insert(input.clone(), vec![SystemTime::UNIX_EPOCH]);
    }
    parser.graph.run(&recipe, inputs_map);
}
