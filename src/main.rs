use std::time::SystemTime;
use std::collections::HashMap;
use rgparse::{Parser, Parameter};
use std::fs;

mod builder;
mod token;
mod lexer;

fn main() {
    let mut parser = Parser::new("A tool for fast incremental builds");
    parser.add_parameter(Parameter::param("--threads", "The number of threads to use during execution").alias("-t").default(&8));
    parser.add_parameter(Parameter::param("--cache", "The cache file to read from and write to. RBuild uses this to figure out when a command has been modified and needs to be re-run.").alias("-c").default(&"rbuild.cache"));
    let args = parser.parse_args();

    // Get the config file and build a graph.
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
    let (mut graph, node_map) = builder::build_graph(&config, args.get("--threads"));

    // Read cache if it exists.
    let cache_path: String = args.get("--cache");
    println!("Reading cache: {}", cache_path);
    if let Ok(cache_bytes) = fs::read(&cache_path) {
        builder::read_hash_cache(&mut graph, &node_map, &cache_bytes);
    };

    // Assemble fetches, i.e. nodes to run, based on command line arguments.
    // Here we can consume the command line arguments.
    let mut fetches = Vec::new();
    for target in args.positional.into_iter().skip(1) {
        fetches.push(*node_map.get(&target).expect(
            &format!("{} is not a valid target", target)
        ));
    }
    // By default, we will build all the targets in the configuration file.
    if fetches.is_empty() {
        fetches = (0..graph.len()).collect();
    }

    // Compile and run the graph
    // TODO: Headers/Source files with no commands may be a huge bottleneck, need to profile.
    // If so, add a fast_execute path to paragraphs.
    if fetches.len() > 0 {
        let recipe = graph.compile(fetches);
        let mut inputs_map = HashMap::with_capacity(recipe.inputs.len());
        for input in &recipe.inputs {
            inputs_map.insert(input.clone(), vec![SystemTime::UNIX_EPOCH]);
        }
        graph.run(&recipe, inputs_map);
    }

    // Write out cache hash after running the graph.
    let mut cache = match fs::File::create(&cache_path) {
        Ok(file) => file,
        Err(what) => panic!("Failed to write cache file ({}):\n\t{}", cache_path, what),
    };
    builder::write_hash_cache(&mut cache, &graph);
}
