use std::fs;
use std::time::SystemTime;
use std::process::Command;
use std::collections::HashMap;
use paragraphs::{Graph, ThreadExecute};

#[derive(Debug)]
pub struct Target {
    path: String,
    run: Vec<Command>,
}

impl Target {
    fn new(path: String, cmds: Vec<Command>) -> Target {
        return Target{path: path, run: cmds};
    }
}

// TODO: Need to log hashes of the commands previously run by this target so it can be rerun as needed.
impl ThreadExecute<SystemTime> for Target {
    fn execute(&mut self, inputs: Vec<&SystemTime>) -> Option<SystemTime> {
        fn get_timestamp(path: &String) -> SystemTime {
            return match fs::metadata(path) {
                Ok(meta) => match meta.modified() {
                    Ok(timestamp) => timestamp,
                    Err(_) => panic!("Could not access timestamp for {}", path)
                },
                Err(_) => SystemTime::UNIX_EPOCH,
            };
        }

        let timestamp = get_timestamp(&self.path);
        let newest_input = inputs.iter().cloned().max().unwrap_or(&SystemTime::UNIX_EPOCH);
        if newest_input > &timestamp {
            for cmd in &mut self.run {
                match cmd.status() {
                    Ok(stat) => {
                        if !stat.success() {
                            println!("Command {:?} exited with status {}", cmd, stat);
                            return None
                        }
                    },
                    Err(what) => {
                        println!("During build of {}, in command {:?}, encountered an error:\n\t{}\n\tDoes the executable specified in this command exist?", self.path, cmd, what);
                        return None;
                    },
                };
            }
        }
        // Return the newest timestamp of all this node's inputs + its own.
        return Some(std::cmp::max(*newest_input, get_timestamp(&self.path)));
    }
}

pub fn build_graph(config: &str, num_threads: usize) -> (Graph<Target, SystemTime>, HashMap<String, usize>) {
    let mut graph = Graph::new(num_threads);
    let mut node_map: HashMap<String, usize> = HashMap::new();
    // Per target variables
    let mut path = String::new();
    let mut inputs = Vec::new();
    let mut cmds = Vec::new();

    for (lineno, line) in config.lines().enumerate() {
        // Each line MUST begin with a keyword describing what that line represents,
        // so we can safely split the string by whitespace here.
        if let Some(keyword) = line.split_whitespace().nth(0) {
            // Skip over the keyword, find the first non-whitespace character,
            // and then compute its index with keyword.len() + position.
            // This gives us the value for the keyword.
            let value = line[match line.chars().skip(keyword.len()).position(|x| !x.is_whitespace()) {
                Some(index) => keyword.len() + index,
                None => panic!("Error: Line {}: '{}' keyword does not specify a value", keyword, lineno),
            }..].trim_end();

            match keyword {
                "path" => {
                    // When we encounter a new path, we first push the old node if it is non-empty.
                    // Then, reset all variables for the target
                    if !path.is_empty() {
                        // TODO: Maybe eliminate the clone here by exposing get/get_mut in Graph.
                        node_map.insert(path.clone(), graph.add(Target::new(path, cmds), inputs));
                        inputs = Vec::new();
                        cmds = Vec::new();
                    }
                    path = String::from(value);
                },
                "dep" => {
                    let input_idx = node_map.get(value).expect(
                        &format!("Error: Line {}: {} specified as a dependency, but did not match any specified paths.", lineno, value)
                    );
                    inputs.push(input_idx.clone());
                },
                "run" => {
                    cmds.push(Command::new(value));
                },
                "arg" => {
                    let last_cmd = cmds.last_mut().expect(
                        &format!("Error: Line {}: Argument specified before command", lineno)
                    );
                    last_cmd.arg(value);
                },
                _ => panic!("Error: Line {}: Unrecognized keyword: '{}'", lineno, keyword)
            }
        };
    }
    // Add the last remaining node and return the graph.
    node_map.insert(path.clone(), graph.add(Target::new(path, cmds), inputs));
    return (graph, node_map);
}
