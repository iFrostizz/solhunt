// A module contains the matching logic to be paired with the ast

use crate::walker::{Finding, Findings};
use ethers_solc::artifacts::ast::{ContractDefinitionPart, SourceUnitPart};
use semver::Version;

#[derive(Debug)]
pub struct Module<F> {
    pub name: String,
    pub findings: Findings,
    pub func: F,
}

#[derive(Debug, Clone)]
pub struct Information {
    pub name: String,
    pub version: Version,
}

impl<F> Module<F>
where
    F: Fn(&SourceUnitPart, &Information) -> Vec<Finding>,
{
    pub fn new(name: impl Into<String>, func: F) -> Module<F> {
        Module {
            name: name.into(),
            findings: Vec::new(),
            func,
        }
    }

    pub fn process_source(&self, source: &SourceUnitPart, info: &Information) -> Vec<Finding> {
        (self.func)(source, info)
    }

    /*pub fn process_def(&self, def: &ContractDefinitionPart, info: &Information) -> Vec<Finding> {
        (self.func)(def, info)
    }*/
}

pub type DynModule = Module<Box<dyn Fn(&SourceUnitPart, &Information) -> Vec<Finding>>>;
