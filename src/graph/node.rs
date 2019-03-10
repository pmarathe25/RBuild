use std::vec::Vec;
use std::collections::HashSet;
use std::time::SystemTime;
use std::fs;

#[derive(Debug)]
pub struct Node<'a> {
    pub path: &'a str,
    pub inputs: HashSet<usize>,
    pub cmds: Vec<&'a str>,
    pub timestamp: SystemTime,
}

impl<'a> Node<'a> {
    pub(crate) fn init(path: &'a str) -> Node<'a> {
        let timestamp = match fs::metadata(path) {
            Ok(meta) => match meta.modified() {
                Ok(timestamp) => timestamp,
                Err(_) => panic!("Could not access timestamp for {}", path)
            },
            Err(_) => SystemTime::UNIX_EPOCH,
        };
        return Node{path: path, inputs: HashSet::new(), cmds: Vec::new(), timestamp: timestamp};
    }
}
