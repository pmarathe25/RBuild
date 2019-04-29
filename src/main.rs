use std::time::SystemTime;
use std::collections::HashMap;
use rgparse::{Parser, Parameter};
use std::fs;

mod target;
mod token;
mod lexer;
mod parser;

fn main() {
    let mut parser = Parser::new("A tool for fast incremental builds");
    parser.add_parameter(Parameter::param("--threads", "The number of threads to use during execution").alias("-t").default(&8));
    parser.add_parameter(Parameter::param("--cache", "The cache file to read from and write to. RBuild uses this to figure out when a command has been modified and needs to be re-run.").alias("-c").default(&"rbuild.cache"));
    let args = parser.parse_args();

    // Parse the config file and build a graph.
    let config_path = match args.positional.get(0) {
        Some(val) => val,
        None => {
            parser.help();
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

    // Read cache into the graph..
    let cache_path: String = args.get("--cache");
    if let Ok(cache_bytes) = fs::read(&cache_path) {
        println!("Reading cache: {}", cache_path);
        target::read_hash_cache(&mut parser.graph, &parser.node_map, &cache_bytes);
    };

    // Assemble fetches, i.e. nodes to run, based on command line arguments.
    // Here we can consume the command line arguments.
    let mut fetches = Vec::new();
    for target in args.positional.into_iter().skip(1) {
        fetches.push(
            match parser.node_map.get(&target) {
                Some(&id) => id,
                None => panic!("{} is not a valid target", target),
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

    // Write out cache hash after running the graph.
    let mut cache = match fs::File::create(&cache_path) {
        Ok(file) => file,
        Err(what) => panic!("Failed to write cache file ({}):\n\t{}", cache_path, what),
    };
    target::write_hash_cache(&mut cache, &parser.graph);
}
