use std::collections::HashMap;
use std::vec::Vec;

mod node;
use crate::graph::node::Node;

pub struct Graph<'a> {
    nodes: Vec<Node<'a>>,
    node_indices: HashMap<&'a str, usize>,
}

impl<'a> Graph<'a> {
    pub fn from_config(config: &'a str) -> Graph<'a> {
        let mut graph = Graph{nodes: Vec::new(), node_indices: HashMap::new()};
        let mut cur_node_id: usize = 0;

        for (lineno, line) in config.lines().enumerate() {
            // Each line MUST begin with a keyword describing what that line represent, so we can safely split the string by whitespace here.
            // This is actually quite efficient, as iterators are lazily evaluated.
            match line.split_whitespace().nth(0) {
                Some(keyword) => {
                    // Skip over the keyword, find the first non-whitespace character, and then compute its index with keyword.len() + position.
                    let value_index = match line.chars().skip(keyword.len()).position(|x| !x.is_whitespace()) {
                        Some(index) => keyword.len() + index,
                        None => panic!("Error: Line {}: '{}' keyword does not specify a value", keyword, lineno),
                    };
                    // This gives us the value for the keyword.
                    let value = &line[value_index..];

                    match keyword {
                        "path" => {
                            cur_node_id = graph.get_node_index(value);
                        },
                        "dep" => {
                            // Add this dependency to cur_node_id
                            let dep_node_id = graph.get_node_index(value);
                            match graph.nodes.get_mut(cur_node_id) {
                                Some(val) => val,
                                None => panic!("Error: Line {}: dep specified before path", lineno),
                            }.inputs.push(dep_node_id);
                        },
                        // Add commands
                        "run" => {
                            match graph.nodes.get_mut(cur_node_id) {
                                Some(val) => val,
                                None => panic!("Error: Line {}: run specified before path", lineno),
                            }.cmds.push(&line[keyword.len()..]);
                        },
                        _ => panic!("Error: Line {}: Unrecognized keyword: '{}'", lineno, keyword)
                    }
                },
                None => ()
            };
        }
        return graph;
    }

    // Gets a node's index based on its path. If the node does not exist the graph, creates one.
    fn get_node_index(&mut self, path: &'a str) -> usize {
        return match self.node_indices.get(path) {
            Some(index) => *index,
            None => {
                let index = self.nodes.len();
                self.nodes.push(Node::init(path));
                self.node_indices.insert(path, index);
                return index;
            }
        };
    }
}

impl<'a> std::fmt::Display for Graph<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for node in &self.nodes {
            f.write_fmt(format_args!("{}\n\tInputs:", node.path))?;
            for input in &node.inputs {
                f.write_fmt(format_args!("\n\t\t{}",
                    match self.nodes.get(*input){
                        Some(node) => node,
                        None => panic!("Error: No node at index {}", input),
                    }.path))?;
            }
            f.write_str("\n\tCommands:")?;
            for cmd in &node.cmds {
                f.write_fmt(format_args!("\n\t\t{}", cmd))?;
            }
            f.write_str("\n")?;
        }
        return Ok(());
    }
}
