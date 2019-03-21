use std::fs;
use std::time::SystemTime;
use std::process::Command;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use paragraphs::{Graph, ThreadExecute};
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct Target {
    path: String,
    // Keeps track of Commands and their hashes based on previous runs.
    run: Vec<(Command, Option<u64>)>,
}

impl Target {
    fn new(path: String, cmds: Vec<(Command, Option<u64>)>) -> Target {
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
        for (cmd, prev_hash_opt) in &mut self.run {
            // We need to rerun a command if either the timestamp of our path is older,
            // OR the hash for the command has changed.
            let current_hash = {
                let mut hasher = DefaultHasher::new();
                format!("{:?}", cmd).hash(&mut hasher);
                hasher.finish()
            };
            if newest_input > &timestamp || match prev_hash_opt {
                // If there is a previous hash, we need to run again if the current hash is different.
                Some(prev_hash) => current_hash != *prev_hash,
                // If there is no previous hash, we must run the command.
                None => true,
            } {
                match cmd.status() {
                    Ok(stat) => {
                        if !stat.success() {
                            println!("Command {:?} exited with status {}", cmd, stat);
                            return None
                        } else {
                            // If the command succeeded, we can update the hash.
                            prev_hash_opt.replace(current_hash);
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
            // Skip over the keyword, then trim leading/trailing whitespace.
            let value = line[keyword.len()..].trim_start().trim_end();
            if value.is_empty() {
                panic!("Error: Line {}: '{}' keyword does not specify a value", keyword, lineno);
            }

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
                    cmds.push((Command::new(value), None));
                },
                "arg" => {
                    let (last_cmd, _) = cmds.last_mut().expect(
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
