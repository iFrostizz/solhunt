// A module contains the matching logic to be paired with the ast

use crate::walker::{Finding, Findings};
use ethers_solc::artifacts::ast::Node;

pub struct Module<F> {
    pub name: String,
    pub findings: Findings,
    pub func: F,
}

impl<F> Module<F>
where
    F: Fn(&Node) -> Option<Finding>,
{
    pub fn new(name: impl Into<String>, func: F) -> Module<F> {
        Module {
            name: name.into(),
            findings: Vec::new(),
            func,
        }
    }

    pub fn process(&self, node: &Node) -> Option<Finding> {
        (self.func)(node)
    }
}
