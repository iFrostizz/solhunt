// Takes a load of modules and walk through the full ast. Should be kind enough to tell bugs

use super::{Findings, Meta, MetaFinding};
use crate::{
    loader::{Information, Loader},
    solidity::get_line_position,
    walker::AllFindings,
};
use ethers_solc::{
    artifacts::{
        ast::{lowfidelity::Ast, SourceUnit},
        visitor::{VisitError, Visitable, Visitor},
    },
    ArtifactId, ConfigurableContractArtifact,
};
use std::collections::HashMap;
use std::{collections::btree_map::BTreeMap, path::PathBuf};

pub struct Walker<V: Visitor<Error = VisitError>> {
    artifact: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
    loader: Loader,
    source_map: BTreeMap<String, Vec<usize>>,
    visitor: V,
}

impl<V: Visitor<Error = VisitError>> Walker<V> {
    pub fn new(
        artifact: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
        loader: Loader,
        source_map: BTreeMap<String, Vec<usize>>,
        visitor: V,
    ) -> Self {
        Walker {
            artifact,
            loader,
            source_map,
            visitor,
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
            // let source = ast.borrow_mut();
            if !ids.contains(&ast.id) {
                ids.push(ast.id);

                let abs_path = id.source.to_str().unwrap().to_string();
                let lines_to_bytes = &source_map.get(&abs_path).unwrap()/*.unwrap_or(&Vec::new())*/;

                // let nodes = &ast.nodes;

                let path = PathBuf::from(&ast.absolute_path);
                let name = path.file_name().unwrap();
                let name = name.to_os_string().into_string().unwrap();
                // .sol is redundant
                let name = name.strip_suffix(".sol").unwrap();

                let info = Information {
                    name: name.to_string(),
                    version: id.version.clone(),
                };

                visit_source(
                    &mut ast,
                    &mut self.visitor,
                    lines_to_bytes,
                    info,
                    &mut all_findings,
                );
            }
        });

        Ok(all_findings)
    }
}

pub fn visit_source<V: Visitor<Error = VisitError>>(
    source: &mut SourceUnit,
    visitor: &mut V,
    lines_to_bytes: &[usize],
    info: Information,
    findings: &mut AllFindings,
) {
    source
        .clone()
        .visit(visitor)
        .expect("ast traversal failed!");

    let file = info.name;

    visitor.0.findings.iter().for_each(|finding| {
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
        findings
            .entry(finding.name.clone())
            .and_modify(|f| f.push(meta_finding.clone()))
            .or_insert(vec![meta_finding]);
    });
}
