use std::env;
use std::string::String;
use std::process::Command;
use std::time::SystemTime;
use std::collections::HashMap;
use paragraphs::{Graph, ThreadExecute};

fn help(executable: &str) {
    println!("Usage: {} RBUILD_FILE [TARGETS...]
    RBUILD_FILE     Path to the rbuild configuration file.
    TARGETS         Zero or more targets to execute.", executable);
    std::process::exit(1);
}

struct Target {
    path: String,
    run: Vec<Command>,
}

impl Target {
    fn new(path: String, cmds: Vec<Command>) -> Target {
        return Target{path: path, run: cmds};
    }
}

impl ThreadExecute<SystemTime> for Target {
    fn execute(&mut self, inputs: Vec<&SystemTime>) -> SystemTime {
        // TODO:
        return SystemTime::UNIX_EPOCH;
    }
}

fn build_graph(config: &str) -> Graph<Target, SystemTime> {
    // TODO: Number of threads should be configurable.
    let mut graph = Graph::new(8);
    let mut node_map: HashMap<&str, usize> = HashMap::new();
    // Per target variables
    let mut path = String::new();
    let mut inputs = Vec::new();
    let mut cmds = Vec::new();

    for (lineno, line) in config.lines().enumerate() {
        // Each line MUST begin with a keyword describing what that line represent, so we can safely split the string by whitespace here.
        // This is actually quite efficient, as iterators are lazily evaluated.
        if let Some(keyword) = line.split_whitespace().nth(0) {
            // Skip over the keyword, find the first non-whitespace character,
            // and then compute its index with keyword.len() + position.
            // This gives us the value for the keyword.
            let value = &line[match line.chars().skip(keyword.len()).position(|x| !x.is_whitespace()) {
                Some(index) => keyword.len() + index,
                None => panic!("Error: Line {}: '{}' keyword does not specify a value", keyword, lineno),
            }..];

            match keyword {
                "path" => {
                    // When we encounter a new path, we first push the old node if it is non-empty.
                    if !path.is_empty() {
                        graph.add(Target::new(path, cmds), inputs);
                    }
                    path = String::from(value);
                },
                "dep" => {
                    let input_idx = node_map.get(value).expect(
                        &format!("Error: Line {}: {} specified as a dependency, but did not match any specified paths.", lineno, value));
                    inputs.push(input_idx.clone());
                },
                "run" => {
                    cmds.push(value);
                },
                _ => panic!("Error: Line {}: Unrecognized keyword: '{}'", lineno, keyword)
            }
        };
    }
    return graph;
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


    // DEBUG:
    println!("Graph contains:\n{}", graph);

    // First argument is the executable name and the second is the rbuild file.
    println!("{} using config file {}", args[0], args[1]);
    for target in args.iter().skip(2) {
        println!("Found target: {}", target);
    }
}
