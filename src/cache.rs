use std::fs;
use std::io::Write;
use paragraphs::Graph;
use std::time::SystemTime;
use std::collections::HashMap;

use crate::target::{HashCommand, Target};

/// Writes a hash cache from the graph to the provided cache_path.
pub(crate) fn write(cache: &mut fs::File, graph: &Graph<Target, SystemTime>) {
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
pub(crate) fn read(graph: &mut Graph<Target, SystemTime>, node_map: &HashMap<String, usize>, cache_bytes: &Vec<u8>) {
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
