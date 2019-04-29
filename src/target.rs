use std::fs;
use std::sync::Arc;
use std::time::SystemTime;
use std::process::Command;
use paragraphs::ThreadExecute;

#[derive(Debug)]
pub struct Target {
    pub(crate) path: String,
    pub(crate) cmds: Vec<Command>,
}

impl Target {
    pub(crate) fn new(path: String, cmds: Vec<Command>) -> Target {
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
        // We only need to rerun commands if the timestamp of our path is older that its inputs.
        if newest_input > timestamp {
            for cmd in &mut self.cmds {
                match cmd.status() {
                    Ok(stat) => {
                        if !stat.success() {
                            println!("Command {:?} exited with status {}", cmd, stat);
                            return None;
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
        return Some(std::cmp::max(newest_input, get_timestamp(&self.path)));
    }
}
