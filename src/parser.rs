use std::iter::Peekable;
use std::process::Command;
use std::collections::{HashMap, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};

use crate::token::Token;
use crate::lexer::Lexer;
use crate::target::{HashCommand, Target};
use paragraphs::Graph;
use std::time::SystemTime;

pub(crate) struct Parser<'a> {
    token_stream: Peekable<Lexer<'a>>,
    pub(crate) graph: Graph<Target, SystemTime>,
    pub(crate) node_map: HashMap<String, usize>,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(config: &'a str, num_threads: usize) -> Parser<'a> {
        return Parser{token_stream: Lexer::new(config).peekable(), graph: Graph::new(num_threads), node_map: HashMap::new()};
    }

    fn parse_node(&mut self) {
        // Read path
        match self.token_stream.next() {
            Some(Token::Path) => (),
            _ => panic!("Expected path keyword")
        }
        let path = match self.token_stream.next() {
            Some(Token::Ident(path)) => path,
            err_val @ _ => panic!("Expected a path, but found {:?}", err_val)
        };

        let mut deps = Vec::new();
        let mut cmds = Vec::new();

        loop {
            // Break out of the loop when either:
            // 1. No tokens remain
            // 2. A path token is seen
            if let None | Some(Token::Path) = self.token_stream.peek() {
                break;
            }
            match self.token_stream.next().unwrap() {
                Token::Run => {
                    // Build a hash as we read the command.
                    let mut hasher = DefaultHasher::new();
                    let mut cmd: Command;

                    // Get executable name
                    match self.token_stream.next() {
                        Some(Token::Ident(exec)) => {
                            exec.hash(&mut hasher);
                            cmd = Command::new(exec);
                        },
                        err_val => panic!("Expected executable after run keyword, but recieved {:?}", err_val),
                    }
                    // Pull in all subsequent Idents as arguments to the command.
                    while let Some(Token::Ident(_)) = self.token_stream.peek() {
                        if let Some(Token::Ident(arg)) = self.token_stream.next() {
                            arg.hash(&mut hasher);
                            cmd.arg(arg);
                        }
                    }

                    let cmd_hash = hasher.finish();
                    cmds.push(HashCommand::new(cmd, cmd_hash))
                },
                Token::Deps => {
                    // Pull in all subsequent Nums
                    while let Some(Token::Num(_)) = self.token_stream.peek() {
                        if let Some(Token::Num(dep)) = self.token_stream.next() {
                            deps.push(dep);
                        }
                    }
                },
                // TODO: Better error messages here by explicit checks for different token types.
                err_val @ _ => panic!("Found {:?}, before keyword", err_val),
            }
        }
        self.node_map.insert(path.clone(), self.graph.add(Target::new(path, cmds), deps));
    }

    pub(crate) fn parse(&mut self) {
        while let Some(_) = self.token_stream.peek() {
            self.parse_node();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Parser};
    use std::collections::HashSet;
    use std::iter::FromIterator;

    fn make_parser<'a>(inp: &'a str) -> Parser<'a> {
        return Parser::new(inp, 8);
    }

    #[test]
    fn can_construct_parser() {
        let _ = make_parser("hi");
    }

    #[test]
    fn can_parse_node() {
        let mut parser = make_parser("  path   /my/path ");
        parser.parse_node();
        assert_eq!(parser.graph.get(0).unwrap().path, "/my/path");
    }

    #[test]
    fn can_parse_multiple_nodes() {
        let mut parser = make_parser("path /my/path/0 path /my/path/1");
        parser.parse();
        assert_eq!(parser.graph.get(0).unwrap().path, "/my/path/0");
        assert_eq!(parser.graph.get(1).unwrap().path, "/my/path/1");
    }

    #[test]
    fn can_parse_multiple_nodes_with_deps() {
        let mut parser = make_parser("path #0 /my/path/0 path /my/path/1 deps 0");
        parser.parse();
        assert_eq!(parser.graph.get(0).unwrap().path, "/my/path/0");
        assert_eq!(parser.graph.get(1).unwrap().path, "/my/path/1");
        // Next we check that if we compile for node 1, node 0 is the recipe input.
        let recipe = parser.graph.compile(&[1]);
        assert_eq!(recipe.inputs, HashSet::from_iter(vec![0 as usize]));
        assert_eq!(recipe.outputs, HashSet::from_iter(vec![1 as usize]));
    }
}
