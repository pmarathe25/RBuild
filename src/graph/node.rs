use std::fs;
use std::vec::Vec;
use std::time::SystemTime;
use std::process::Command;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) struct NodeCommand {
    pub executable: String,
    pub args: Vec<String>,
}

impl NodeCommand {
    pub(crate) fn new(executable: &str) -> NodeCommand {
        return NodeCommand{executable: executable.to_string(), args: Vec::new()};
    }
}

#[derive(Debug)]
pub(crate) struct Node<'a> {
    pub path: &'a str,
    pub cmds: Vec<NodeCommand>,
    pub timestamp: SystemTime,
}

impl<'a> Node<'a> {
    pub(crate) fn init(path: &'a str) -> Node<'a> {
        let mut node = Node{path: path, cmds: Vec::new(), timestamp: SystemTime::UNIX_EPOCH};
        node.update_timestamp();
        return node;
    }

    pub(crate) fn update_timestamp(&mut self) {
        self.timestamp = match fs::metadata(self.path) {
            Ok(meta) => match meta.modified() {
                Ok(timestamp) => timestamp,
                Err(_) => panic!("Could not access timestamp for {}", self.path)
            },
            Err(_) => SystemTime::UNIX_EPOCH,
        };
    }

    pub(crate) fn execute(&mut self) {
        for cmd in &mut self.cmds {
            Command::new(&cmd.executable).args(&cmd.args).spawn();
        }
        self.update_timestamp();
    }
}
