use std::fs;
use std::sync::Arc;
use std::time::SystemTime;
use std::process::Command;
use paragraphs::ThreadExecute;

#[derive(Debug)]
pub struct Target {
    pub(crate) path: String,
    pub(crate) cmds: Vec<Command>,
    pub(crate) always_cmds: Vec<Command>,
}

impl Target {
    pub(crate) fn new(path: String, cmds: Vec<Command>, always_cmds: Vec<Command>) -> Target {
        return Target{path: path, cmds: cmds, always_cmds: always_cmds};
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

        fn check_cmd(cmd: &mut Command, path: &String) -> bool {
            match cmd.status() {
                Ok(stat) => {
                    if !stat.success() {
                        println!("Command {:?} exited with status {}", cmd, stat);
                        return false;
                    }
                },
                Err(what) => {
                    println!("During build of {}, in command {:?}, encountered an error:\n\t{}\n\tDoes the executable specified in this command exist?", path, cmd, what);
                    return false;
                },
            };
            return true;
        }

        // We only need to rerun commands if the timestamp of our path is older that its inputs.
        if newest_input > timestamp {
            for cmd in &mut self.cmds {
                if !check_cmd(cmd, &self.path) {
                    return None;
                }
            }
        }

        for cmd in &mut self.always_cmds {
            if !check_cmd(cmd, &self.path) {
                return None;
            }
        }

        // Return the newest timestamp of all this node's inputs + its own.
        return Some(std::cmp::max(newest_input, get_timestamp(&self.path)));
    }
}
