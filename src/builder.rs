use std::fs;
use std::time::SystemTime;
use std::process::Command;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use paragraphs::{Graph, ThreadExecute};
use std::hash::{Hash, Hasher};
use std::io::Write;

#[derive(Debug)]
pub struct Target {
    path: String,
    // Keeps track of Commands and their hashes based on previous runs.
    cmds: Vec<(Command, Option<u64>)>,
}

impl Target {
    fn new(path: String, cmds: Vec<(Command, Option<u64>)>) -> Target {
        return Target{path: path, cmds: cmds};
    }
}

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
        for (cmd, prev_hash_opt) in &mut self.cmds {
            // We need to rerun a command if either the timestamp of our path is older,
            // OR the hash for the command has changed.
            let current_hash = {
                let mut hasher = DefaultHasher::new();
                format!("{:?}", cmd).hash(&mut hasher);
                hasher.finish()
            };
            if newest_input > &timestamp || match prev_hash_opt {
                // If there is a previous hash, we need to run again if the current hash
                // is different.
                Some(prev_hash) => current_hash != *prev_hash,
                // If there is no previous hash, we must run the command.
                None => true,
            } {
                // DEBUG:
                println!("Running {}", self.path);
                if newest_input > &timestamp {
                    println!("\tTimestamp {:?} is older than newest input {:?}", timestamp, newest_input);
                } else if let Some(prev_hash) = prev_hash_opt {
                    println!("\tPrevious hash: {} does not match current hash: {}", prev_hash, current_hash);
                } else {
                    println!("\tNo previous hash found. Current hash is {}", current_hash);
                }


                match cmd.status() {
                    Ok(stat) => {
                        if !stat.success() {
                            println!("Command {:?} exited with status {}", cmd, stat);
                            return None
                        } else {
                            // If the command succeeded, we can update the hash.
                            // DEBUG:
                            println!("Command successfully ran. New hash: {}", current_hash);
                            prev_hash_opt.replace(current_hash);
                        }
                    },
                    Err(what) => {
                        println!("During build of {}, in command {:?}, encountered an error:\n\t{}\n\tDoes the executable specified in this command exist?", self.path, cmd, what);
                        return None;
                    },
                };
            } else {
                // DEBUG:
                println!("Skipping {:?} in  {}", cmd, self.path);
            }
        }
        // Return the newest timestamp of all this node's inputs + its own.
        return Some(std::cmp::max(*newest_input, get_timestamp(&self.path)));
    }
}

/// Writes a hash cache from the graph to the provided cache_path.
pub fn write_hash_cache(cache: &mut fs::File, graph: &Graph<Target, SystemTime>) {
    for node in graph {
        // No need to cache for nodes that have never been run before,
        // so check for the presence of hashes.
        if let Some((_, Some(_))) = node.cmds.first() {
            // Format is:
            // [Path length][Path][Num Cmds][Cmds]...
            // Using little endian for integers.
            cache.write_all(&(node.path.len() as u64).to_le_bytes()).unwrap();
            cache.write_all(node.path.as_bytes()).unwrap();
            cache.write_all(&(node.cmds.len() as u64).to_le_bytes()).unwrap();
            for (_, hash_opt) in &node.cmds {
                cache.write_all(&hash_opt.unwrap().to_le_bytes()).unwrap();
            }
        }
    }
}

// TODO: Docstring
pub fn read_hash_cache(graph: &mut Graph<Target, SystemTime>, node_map: &HashMap<String, usize>, cache_bytes: &Vec<u8>) {
    let mut offset = 0;
    // Memory for storing u64 values.
    let mut u64_bytes = [0; 8];

    while offset < cache_bytes.len() {
        // Get length of path, then path itself..
        u64_bytes.copy_from_slice(&cache_bytes[offset..(offset + 8)]);
        let path_size = u64::from_le_bytes(u64_bytes) as usize;
        offset += 8;
        let path = std::str::from_utf8(&cache_bytes[(offset)..(offset + path_size)]).unwrap();
        offset += path_size;
        // If a path is missing from the graph, we just ignore it.
        if let Some(target_id) = node_map.get(path) {
            let target_node = graph.get_mut(*target_id).expect(
                &format!("Invalid entry in node map: {{{}: {}}}", path, target_id)
            );
            // Get number of commands, then command hashes.
            u64_bytes.copy_from_slice(&cache_bytes[offset..(offset + 8)]);
            let num_cmds = u64::from_le_bytes(u64_bytes) as usize;
            offset += 8;
            for (_, hash_opt) in target_node.cmds.iter_mut().take(num_cmds) {
                u64_bytes.copy_from_slice(&cache_bytes[offset..(offset + 8)]);
                let cmd_hash = u64::from_le_bytes(u64_bytes);
                offset += 8;
                hash_opt.replace(cmd_hash);
            }
        }
    }
}

// TODO: Change this so that node_map uses the hashes of the paths instead.
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
