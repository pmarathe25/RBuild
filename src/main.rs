use std::time::SystemTime;
use std::collections::HashMap;
use rgparse::{Parser, Parameter};
use std::fs;

mod builder;

fn main() {
    let mut parser = Parser::new("A tool for fast incremental builds");
    parser.add_parameter(Parameter::param("--threads", "The number of threads to use during execution").alias("-t"));
    let args = parser.parse_args();

    let config_path = args.positional.get(0).expect("No configuration file provided.");
    let config = match std::fs::read_to_string(&config_path) {
        Ok(file) => file,
        Err(what) => panic!("Could not open provided configuration file {}: {}", config_path, what),
    };

    let (mut graph, node_map) = builder::build_graph(&config, args.get("--threads").unwrap_or(8));

    // TODO: improve read_hash_cache logic.
    // DEBUG:
    println!("Reading cache: {}", "rbuild.cache");
    let cache_bytes = match fs::read("rbuild.cache") {
        Ok(file) => file,
        Err(what) => panic!("Failed to open cache file:\n\t{}", what),
    };
    builder::read_hash_cache(&mut graph, &node_map, &cache_bytes);

    let mut fetches = Vec::new();
    for target in args.positional.into_iter().skip(1) {
        fetches.push(node_map.get(&target).expect(
            &format!("{} is not a valid target", target)
        ));
    }

    if fetches.len() > 0 {
        let recipe = graph.compile(fetches);
        let mut inputs_map = HashMap::with_capacity(recipe.inputs.len());
        for input in &recipe.inputs {
            inputs_map.insert(input.clone(), vec![SystemTime::UNIX_EPOCH]);
        }
        graph.run(&recipe, inputs_map);
    }
    // Write out cache hash after running the graph.
    // TODO: Unhardcode this.
    let mut cache = match fs::File::create("rbuild.cache") {
        Ok(file) => file,
        Err(what) => panic!("Failed to open cache file:\n\t{}", what),
    };

    builder::write_hash_cache(&mut cache, &graph);
}
