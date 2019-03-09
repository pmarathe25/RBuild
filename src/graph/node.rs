use std::vec::Vec;
use std::time::SystemTime;
use std::fs;

#[derive(Debug)]
pub struct Node<'a> {
    pub path: &'a str,
    pub inputs: Vec<usize>,
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
        return Node{path: path, inputs: Vec::new(), cmds: Vec::new(), timestamp: timestamp};
    }
}
