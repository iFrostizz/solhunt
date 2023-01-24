// A module contains the matching logic to be paired with the ast

use std::collections::HashMap;

use crate::walker::{AllFindings, Finding, Findings};
use ethers_solc::artifacts::ast::SourceUnitPart;
use semver::Version;

#[derive(Debug, Default, Clone)]
pub struct ModuleFindings {
    pub name: String,
    pub findings: Vec<Finding>,
}

#[derive(Debug)]
pub struct Module<F> {
    pub name: String,
    pub findings: AllFindings,
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
            findings: HashMap::new(),
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
