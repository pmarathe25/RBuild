use std::fs;
use std::vec::Vec;
use std::time::SystemTime;
use std::sync::{Arc, Mutex};
use std::process::Command;

// #[derive(Clone, Debug)]
// pub(crate) struct Command {
//     pub executable: String,
//     pub args: Vec<String>,
// }
//
// impl Command {
//     pub(crate) fn new(executable: &str) -> Command {
//         return Command{executable: executable.to_string(), args: Vec::new()};
//     }
//
//     pub(crate) fn execute(&self) {
//         std::process::Command::new(&self.executable).args(&self.args).spawn();
//     }
// }

#[derive(Debug)]
pub(crate) struct Node<'a> {
    pub path: &'a str,
    pub cmds: Arc<Mutex<Vec<Command>>>,
    pub timestamp: SystemTime,
}

impl<'a> Node<'a> {
    pub(crate) fn init(path: &'a str) -> Node<'a> {
        let mut node = Node{path: path, cmds: Arc::new(Mutex::new(Vec::new())), timestamp: SystemTime::UNIX_EPOCH};
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
}
