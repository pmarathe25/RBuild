use std::vec::Vec;

#[derive(Debug)]
pub(crate) struct PathNode<'a> {
    path: &'a str,
    pub(crate) inputs: Vec<&'a str>,
    pub(crate) cmds: Vec<&'a str>,
}

impl<'a> std::fmt::Display for PathNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(Node: {}\n\tInputs: {:?}\n\tRun: {:?})", self.path, self.inputs, self.cmds)
    }
}

impl<'a> PathNode<'a> {
    pub(crate) fn init(path: &'a str) -> PathNode<'a> {
        return PathNode{path: path, inputs: vec!(), cmds: vec!()};
    }
}
