use std::env;
use std::collections::HashMap;
use std::vec::Vec;
use std::cell::Cell;
use std::string::String;

fn help(executable: &str) {
    println!("Usage: {} RBUILD_FILE [TARGETS...]
    RBUILD_FILE     Path to the rbuild configuration file.
    TARGETS         Zero or more targets to execute.", executable);
    std::process::exit(1);
}

struct PathNode<'a> {
    path: &'a str,
    inputs: Cell<Vec<&'a PathNode<'a>>>,
    cmds: Vec<&'a str>,
}

impl<'a> std::fmt::Debug for PathNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut inp_string = String::new();
        unsafe {
            for inp in &*(self.inputs.as_ptr()) {
                inp_string = inp_string + inp.path + ", ";
            }
        }
        write!(f, "(Node: {}\n\tInputs: {}\n\tRun: {:?})", self.path, inp_string, self.cmds)
    }
}

impl<'a> PathNode<'a> {
    fn init(path: &'a str) -> PathNode<'a> {
        return PathNode{path: path, inputs: Cell::new(vec!()), cmds: vec!()};
    }
}

fn parse_config_file(config_file: &str) {
    let mut graph = HashMap::new();
    let mut active_node: &str = "";
    let mut path_deps: HashMap<&str, Vec<&str>> = HashMap::new();

    for (lineno, line) in config_file.lines().enumerate() {
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
                        match graph.insert(active_node, PathNode::init(active_node)) {
                            None => (),
                            _ => panic!("Error: On line {}: path {} specified more than once", lineno, value),
                        };
                        path_deps.insert(active_node, vec!());
                    },
                    "dep" => {
                        // Push this dependency to the path_deps map.
                        match path_deps.get_mut(active_node) {
                            Some(val) => val,
                            None => panic!("Error: On line {}: dep specified before path", lineno),
                        }.push(value);
                    },
                    "run" => {
                        // We can add commands directly to the node.
                        match graph.get_mut(active_node) {
                            Some(val) => val,
                            None => panic!("Error: On line {}: run specified before path", lineno),
                        }.cmds.push(&line[keyword.len()..]);
                    },
                    _ => panic!("Error: On line {}: Unrecognized keyword: '{}'", lineno, keyword)
                }
            },
            None => ()
        };
    }

    // Walk over the path dependencies and update the graph.
    for (path, deps) in &path_deps {
        // TODO: Handle error case here
        let active_node = graph.get(path).unwrap();
        for dep in deps {
            let dep_node = graph.get(dep).unwrap();
            unsafe {
                (*active_node.inputs.as_ptr()).push(dep_node);
            }
        }
    }

    // DEBUG:
    println!("Graph contains: {:?}\n", graph);
    println!("Path dependency map contains: {:?}\n", path_deps)
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
