use std::time::SystemTime;
use std::collections::HashMap;
use rgparse::{Parser, Parameter};

mod builder;
use builder::build_graph;

fn main() {
    let mut parser = Parser::new("A tool for fast incremental builds");
    parser.add_parameter(Parameter::param("--threads", "The number of threads to use during execution").alias("-t"));
    let args = parser.parse_args();

    let config_path = args.positional.get(0).expect("No configuration file provided.");
    let config = match std::fs::read_to_string(&config_path) {
        Err(what) => panic!("Could not open provided configuration file {}: {}", config_path, what),
        Ok(file) => file,
    };

    let (mut graph, node_map) = build_graph(&config, args.get("--threads").unwrap_or(8));

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
}
