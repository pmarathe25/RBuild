use std::fmt::Display;
use std::collections::{HashMap, HashSet};
use std::vec::Vec;

mod node;
use crate::graph::node::Node;

#[derive(Debug)]
pub struct Graph<'a> {
    pub nodes: Vec<Node<'a>>,
    node_inputs: Vec<HashSet<usize>>,
    node_outputs: Vec<HashSet<usize>>,
    node_indices: HashMap<&'a str, usize>,
}

impl<'a> Graph<'a> {
    pub fn from_config(config: &'a str) -> Graph<'a> {
        // Gets a node's index based on its path. If the node does not exist the graph, creates one.
        fn get_or_insert<'a>(graph: &mut Graph<'a>, path: &'a str) -> usize {
            return match graph.node_indices.get(path) {
                Some(index) => *index,
                None => {
                    let index = graph.nodes.len();
                    graph.nodes.push(Node::init(path));
                    graph.node_inputs.push(HashSet::new());
                    graph.node_outputs.push(HashSet::new());
                    graph.node_indices.insert(path, index);
                    return index;
                }
            };
        }

        let mut graph = Graph{nodes: Vec::new(), node_inputs: Vec::new(), node_outputs: Vec::new(), node_indices: HashMap::new()};
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
                            cur_node_id = get_or_insert(&mut graph, value);
                        },
                        "dep" => {
                            // Add node inputs and outputs.
                            let dep_node_id = get_or_insert(&mut graph, value);
                            match graph.node_inputs.get_mut(cur_node_id) {
                                Some(val) => val,
                                None => panic!("Error: Line {}: dep specified before path", lineno),
                            }.insert(dep_node_id);
                            match graph.node_outputs.get_mut(dep_node_id) {
                                Some(val) => val,
                                None => panic!("Error: Line {}: dep specified before path", lineno),
                            }.insert(cur_node_id);
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

    /// Given a set of node indices, gets all dependencies of those nodes,
    /// including nested dependencies.
    pub fn get_deps<T>(&self, output_indices: T) -> (HashSet<usize>, HashSet<usize>)
        where T: Iterator<Item=usize> + Clone {
        // Gets all the dependencies for a single output node and adds them to the deps HashSet.
        // Also tracks which nodes have no dependencies.
        fn get_single_dep(graph: &Graph, node_index: usize, deps: &mut HashSet<usize>, depless: &mut HashSet<usize>) {
            let inputs = match graph.node_inputs.get(node_index) {
                Some(node) => node,
                None => panic!("Could not find node at index {}", node_index),
            };
            for input in inputs {
                deps.insert(*input);
                get_single_dep(graph, *input, deps, depless);
            }
            if inputs.len() == 0 {
                depless.insert(node_index);
            }
        }

        let mut all_deps: HashSet<usize> = output_indices.clone().collect();
        let mut depless = HashSet::new();
        for out in output_indices {
            get_single_dep(&self, out, &mut all_deps, &mut depless);
        }
        return (all_deps, depless);
    }

    /// Given a path, gets the index of the corresponding node.
    pub fn get_index(&self, path: &'a str) -> Option<&usize> {
        return self.node_indices.get(path);
    }
}

impl<'a> Display for Graph<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (index, node) in self.nodes.iter().enumerate() {
            f.write_fmt(format_args!("{}\n\tInputs:", node.path))?;
            for input in self.node_inputs.get(index).unwrap() {
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
