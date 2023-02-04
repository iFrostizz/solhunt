// Takes a load of modules and walk through the full ast. Should be kind enough to tell bugs

use super::{Finding, Meta, MetaFinding};
use crate::{loader::Information, solidity::get_line_position, walker::AllFindings};
use ethers_solc::{
    artifacts::{
        ast::{lowfidelity::Ast, SourceUnit},
        visitor::{Visitable, Visitor},
    },
    ArtifactId, ConfigurableContractArtifact,
};
use std::collections::HashMap;
use std::{collections::btree_map::BTreeMap, path::PathBuf};

pub struct Walker {
    artifact: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
    source_map: BTreeMap<String, Vec<usize>>,
    visitors: Vec<Box<dyn Visitor<Vec<Finding>>>>,
}

impl Walker {
    pub fn new(
        artifact: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
        source_map: BTreeMap<String, Vec<usize>>,
        visitors: Vec<Box<dyn Visitor<Vec<Finding>>>>,
    ) -> Self {
        Walker {
            artifact,
            source_map,
            visitors,
        }
    }

    // For analyzing a syntax tree, we need an AST "walker" — an object to facilitate the traversal of the tree.
    // The ast module offers two walkers:
    // - ast.NodeVisitor (doesn't allow modification to the input tree)
    // - ast.NodeTransformer (allows modification)
    pub fn traverse(&mut self) -> eyre::Result<AllFindings> {
        let mut all_findings: AllFindings = HashMap::new();

        let mut ids: Vec<usize> = Vec::new();
        let source_map = &self.source_map.clone();

        self.artifact.iter().for_each(|(id, art)| {
            let unique_id = id.identifier();

            let ast: Ast = art
                .ast
                .as_ref()
                .unwrap_or_else(|| panic!("no ast found for {unique_id}"))
                .clone();

            let mut ast: SourceUnit = ast.to_typed();

            // dedup same sources
            // TODO: is that bug from the ast ?
            if !ids.contains(&ast.id) {
                ids.push(ast.id);

                let abs_path = id.source.to_str().unwrap().to_string();
                let lines_to_bytes = source_map.get(&abs_path).unwrap_or(&Vec::new()).clone();

                let path = PathBuf::from(&ast.absolute_path);
                let name = path.file_name().unwrap();
                let name = name.to_os_string().into_string().unwrap();
                // .sol is redundant
                let name = name.strip_suffix(".sol").unwrap();

                let info = Information {
                    name: name.to_string(),
                    version: id.version.clone(),
                };

                self.visitors.iter_mut().for_each(|visitor| {
                    visit_source::<Vec<Finding>>(
                        &mut ast,
                        visitor,
                        &lines_to_bytes,
                        info.clone(),
                        &mut all_findings,
                    );
                });
            }
        });

        Ok(all_findings)
    }
}

pub fn visit_source<D>(
    source: &mut SourceUnit,
    visitor: &mut Box<dyn Visitor<Vec<Finding>>>,
    lines_to_bytes: &[usize],
    info: Information,
    findings: &mut AllFindings,
) {
    source
        .clone()
        .visit(visitor.as_mut())
        .expect("ast traversal failed!");

    let file = info.name;

    let data = visitor.shared_data();

    data.iter().for_each(|finding| {
        let meta_finding = MetaFinding {
            finding: finding.clone(),
            meta: Meta {
                file: file.clone(),
                line: finding
                    .src
                    .clone()
                    .map(|src| get_line_position(&src, lines_to_bytes) as u32),
            },
        };
        std::collections::hash_map::Entry::or_insert(
            findings
                .entry(finding.name.clone())
                .and_modify(|f| f.push(meta_finding.clone())),
            vec![meta_finding],
        );
    });
}
