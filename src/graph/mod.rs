use std::fmt::Display;
use std::collections::{HashMap, HashSet};
use std::vec::Vec;

mod node;
use crate::graph::node::Node;

pub struct Graph<'a> {
    pub nodes: Vec<Node<'a>>,
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
                    graph.node_indices.insert(path, index);
                    return index;
                }
            };
        }

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
                            cur_node_id = get_or_insert(&mut graph, value);
                        },
                        "dep" => {
                            // Add this dependency to cur_node_id
                            let dep_node_id = get_or_insert(&mut graph, value);
                            match graph.nodes.get_mut(cur_node_id) {
                                Some(val) => val,
                                None => panic!("Error: Line {}: dep specified before path", lineno),
                            }.inputs.insert(dep_node_id);
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

    // TODO: Construct subgraphs for each individual target specified, then combine them bottom up.
    // Gets a subgraph with the specified output indices.
    // A subgraph is defined by a stack of layers, where each
    // layer contains one or more node indices.
    // # Arguments
    //
    // * `output_indices` - An iterable over indices of type usize
    pub fn get_subgraph<T>(&self, output_indices: T) -> Vec<HashSet<usize>>
        where T: Iterator<Item=usize> {
        // Given a layer, i.e. a HashSet of indices, gets a new HashSet containing all its inputs
        fn get_layer_inputs(graph: &Graph, layer: &HashSet<usize>) -> HashSet<usize> {
            let mut next_layer = HashSet::new();
            for index in layer {
                let inputs = &match graph.nodes.get(*index) {
                    Some(node) => node,
                    None => panic!("Could not find node at index {}", index)
                }.inputs;
                for inp in inputs {
                    next_layer.insert(*inp);
                }
            }
            return next_layer;
        }

        let mut subgraph: Vec<HashSet<usize>> = Vec::new();
        // Walk over each layer of the subgraph, and its inputs to the layer stack.
        // This loop continues until we reach a layer that's empty.
        let mut layer: HashSet<usize> = output_indices.collect();
        while layer.len() > 0 {
            subgraph.push(layer);
            layer = get_layer_inputs(self, match subgraph.last() {
                Some(elem) => elem,
                None => panic!("Could not retrieve last layer of subgraph.")
            });
        }
        // TODO: Optimize graph by shifting nodes into lower layers when they have no dependecies there.

        return subgraph;
    }

    /// Given a set of node indices, gets all dependencies of those nodes, including nested dependencies.
    pub fn get_all_deps<T>(&self, output_indices: T) -> HashSet<usize>
        where T: Iterator<Item=usize>+Clone {

        let mut all_deps = HashSet::new();
        for out in output_indices.clone() {
            let inputs = match self.nodes.get(out) {
                Some(node) => node,
                None => panic!(),
            }.inputs.iter().cloned();
            let mut deps = self.get_all_deps(inputs);

            all_deps = all_deps.union(&deps).cloned().collect();
        }
        return all_deps.union(&output_indices.collect()).cloned().collect();
    }

    // Given a path, gets the index of the corresponding node.
    pub fn get_index(&self, path: &'a str) -> Option<&usize> {
        return self.node_indices.get(path);
    }
}

impl<'a> Display for Graph<'a> {
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
