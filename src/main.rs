use std::env;

fn help(executable: &str) {
    println!("Usage: {} RBUILD_FILE [TARGETS...]
    RBUILD_FILE     Path to the rbuild configuration file.
    TARGETS         Zero or more targets to execute.", executable);
    std::process::exit(1);
}

#[derive(Debug)]
struct PathNode<'a> {
    path: &'a str,
    inputs: std::vec::Vec<&'a PathNode<'a>>,
    cmds: std::vec::Vec<&'a str>,
}

impl<'a> PathNode<'a> {
    fn init(path: &'a str) -> PathNode<'a> {
        return PathNode{path: path, inputs: vec!(), cmds: vec!()};
    }
}


fn parse_config_file(config_file: &str) {
    let mut graph = std::collections::HashMap::new();
    let mut active_node: &str = "";

    for (lineno, line) in config_file.lines().enumerate() {
        // TODO: Generate nodes w/ cmds but not inputs. Place inputs in string map, then update the nodes with references to their inputs.

        // Each line MUST begin with a keyword describing what that line represent, so we can safely split the string by whitespace here.
        // This is actually quite efficient, as iterators are lazily evaluated.
        match line.split_whitespace().nth(0) {
            Some(keyword) => {
                // Skip over the keyword, find the first non-whitespace character, and then compute its index with keyword.len() + position.
                let value_index = match line.chars().skip(keyword.len()).position(|x| !x.is_whitespace()) {
                    Some(index) => keyword.len() + index,
                    None => panic!("Error: On line {}: '{}' keyword does not specify a value", keyword, lineno),
                };
                // This gives us the value for the keyword.
                let value = &line[value_index..];

                match keyword {
                    "path" => {
                        active_node = value;
                        // DEBUG:
                        println!("Active node is: {}", active_node);
                        if !graph.contains_key(active_node) {
                            graph.insert(active_node, PathNode::init(active_node));
                        }
                    },
                    "dep" => {
                        // graph.get_mut(active_node).unwrap();
                    },
                    "run" => {
                        graph.get_mut(active_node).unwrap().cmds.push(&line[keyword.len()..]);
                    },
                    _ => panic!("Error: On line {}: Unrecognized keyword: '{}'", lineno, keyword)
                }
            },
            None => ()
        };
    }

    // DEBUG:
    println!("Graph contains: {:?}", graph);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        help(&args[0]);
    }

    let config_file = match std::fs::read_to_string(&args[1]) {
        Err(what) => panic!("Could not open provided configuration file {}: {}", args[1], what),
        Ok(file) => file,
    };

    parse_config_file(&config_file);

    // First argument is the executable name and the second is the rbuild file.
    println!("{} using config file {}", args[0], args[1]);
    for target in args.iter().skip(2) {
        println!("Found target: {}", target);
    }
}
