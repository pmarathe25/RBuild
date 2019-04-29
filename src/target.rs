use std::fs;
use std::sync::Arc;
use std::time::SystemTime;
use std::process::Command;
use paragraphs::ThreadExecute;

// Keeps track of a Command, hash of the current run, and optional hash from previous cached runs.
#[derive(Debug)]
pub(crate) struct HashCommand {
    command: Command,
    pub(crate) hash: u64,
    pub(crate) cached_hash: Option<u64>,
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
