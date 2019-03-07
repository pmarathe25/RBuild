use std::collections::HashMap;

mod node;
use crate::graph::node::PathNode;

#[derive(Debug)]
pub(crate) struct Graph<'a> {
    nodes: HashMap<&'a str, PathNode<'a>>,
}

impl<'a> Graph<'a> {
    pub(crate) fn from_config(config: &'a str) -> Graph<'a> {
        let mut nodes = HashMap::new();
        let mut active_node: &str = "";

        for (lineno, line) in config.lines().enumerate() {
            // Each line MUST begin with a keyword describing what that line represent, so we can safely split the string by whitespace here.
            // This is actually quite efficient, as iterators are lazily evaluated.
            match line.split_whitespace().nth(0) {
                Some(keyword) => {
                    // Skip over the keyword, find the first non-whitespace character, and then compute its index with keyword.len() + position.
                    let value_index = match line.chars().skip(keyword.len()).position(|x| !x.is_whitespace()) {
                        Some(index) => keyword.len() + index,
                        None => panic!("Error: On line {}: '{}' keyword does not specify a value", keyword, lineno),
                    };
                    // This gives us the value for the keyword.
                    let value = &line[value_index..];

                    match keyword {
                        "path" => {
                            active_node = value;
                            match nodes.insert(active_node, PathNode::init(active_node)) {
                                None => (),
                                _ => panic!("Error: On line {}: path {} specified more than once", lineno, value),
                            };
                        },
                        // Add dependencies
                        "dep" => {
                            match nodes.get_mut(active_node) {
                                Some(val) => val,
                                None => panic!("Error: On line {}: dep specified before path", lineno),
                            }.inputs.push(value);
                        },
                        // Add commands
                        "run" => {
                            match nodes.get_mut(active_node) {
                                Some(val) => val,
                                None => panic!("Error: On line {}: run specified before path", lineno),
                            }.cmds.push(&line[keyword.len()..]);
                        },
                        _ => panic!("Error: On line {}: Unrecognized keyword: '{}'", lineno, keyword)
                    }
                },
                None => ()
            };
        }
        return Graph{nodes: nodes};
    }
}
