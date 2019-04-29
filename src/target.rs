use std::fs;
use std::sync::Arc;
use std::time::SystemTime;
use std::process::Command;
use std::collections::HashMap;
use paragraphs::{Graph, ThreadExecute};
use std::io::Write;

// Keeps track of a Command, hash of the current run, and optional hash from previous cached runs.
#[derive(Debug)]
pub(crate) struct HashCommand {
    command: Command,
    hash: u64,
    cached_hash: Option<u64>,
}

impl HashCommand {
    pub(crate) fn new(command: Command, hash: u64) -> HashCommand {
        return HashCommand{command: command, hash: hash, cached_hash: None};
    }
}

#[derive(Debug)]
pub struct Target {
    pub(crate) path: String,
    pub(crate) cmds: Vec<HashCommand>,
}

impl Target {
    pub(crate) fn new(path: String, cmds: Vec<HashCommand>) -> Target {
        return Target{path: path, cmds: cmds};
    }
}

impl ThreadExecute<SystemTime> for Target {
    fn execute(&mut self, inputs: Vec<Arc<SystemTime>>) -> Option<SystemTime> {
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
        let newest_input = *(inputs.iter().cloned().max().unwrap_or(Arc::new(SystemTime::UNIX_EPOCH)));
        for hash_cmd in &mut self.cmds {
            // We need to rerun a command if either the timestamp of our path is older,
            // OR the hash for the command has changed.
            if newest_input > timestamp || match hash_cmd.cached_hash {
                // If there is a previous hash, we need to run again if the current hash
                // is different.
                Some(cached) => hash_cmd.hash != cached,
                // If there is no previous hash, we must run the command.
                None => true,
            } {
                match hash_cmd.command.status() {
                    Ok(stat) => {
                        if !stat.success() {
                            println!("Command {:?} exited with status {}", hash_cmd, stat);
                            return None;
                        }
                    },
                    Err(what) => {
                        println!("During build of {}, in command {:?}, encountered an error:\n\t{}\n\tDoes the executable specified in this command exist?", self.path, hash_cmd, what);
                        return None;
                    },
                };
            }
        }
        // Return the newest timestamp of all this node's inputs + its own.
        return Some(std::cmp::max(newest_input, get_timestamp(&self.path)));
    }
}

/// Writes a hash cache from the graph to the provided cache_path.
pub fn write_hash_cache(cache: &mut fs::File, graph: &Graph<Target, SystemTime>) {
    let mut raw_cache = Vec::new();
    // Only cache when a target has commands.
    for target in graph.into_iter().filter(|target| !target.cmds.is_empty()) {
        // Format is:
        // [Path length][Path][Num Cmds][Cmds]...
        // Using little endian for integers.
        raw_cache.extend_from_slice(&(target.path.len() as u64).to_le_bytes());
        raw_cache.extend_from_slice(target.path.as_bytes());
        raw_cache.extend_from_slice(&(target.cmds.len() as u64).to_le_bytes());
        for HashCommand{hash, ..} in &target.cmds {
            raw_cache.extend_from_slice(&hash.to_le_bytes());
        }
    }
    cache.write(raw_cache.as_slice()).unwrap();
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
            let target_node = match graph.get_mut(*target_id) {
                Some(id) => id,
                None => panic!("Invalid entry in node map: {{{}: {}}}", path, target_id)
            };
            // Get number of commands, then command hashes.
            u64_bytes.copy_from_slice(&cache_bytes[offset..(offset + 8)]);
            let num_cmds = u64::from_le_bytes(u64_bytes) as usize;
            offset += 8;
            for HashCommand{cached_hash, ..} in target_node.cmds.iter_mut().take(num_cmds) {
                u64_bytes.copy_from_slice(&cache_bytes[offset..(offset + 8)]);
                cached_hash.replace(u64::from_le_bytes(u64_bytes));
                offset += 8;
            }
        }
    }
}
