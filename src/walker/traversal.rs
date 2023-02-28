// Takes a load of modules and walk through the full ast. Should be kind enough to tell bugs

use super::{Finding, Meta, MetaFinding};
use crate::{
    loader::Information,
    solidity::{get_finding_content, get_position},
    walker::AllFindings,
};
use ethers_solc::{
    artifacts::{
        ast::{lowfidelity::Ast, SourceLocation},
        lowfidelity::TypedAst,
        visitor::{Visitable, Visitor},
    },
    ArtifactId, ConfigurableContractArtifact,
};
use std::collections::HashMap;
use std::{collections::btree_map::BTreeMap, path::PathBuf};

pub struct Walker {
    artifact: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
    source_map: BTreeMap<String, (String, Vec<usize>)>,
    visitors: Vec<Box<dyn Visitor<Vec<Finding>>>>,
    root_abs_path: PathBuf,
}

impl Walker {
    pub fn new(
        artifact: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
        source_map: BTreeMap<String, (String, Vec<usize>)>,
        visitors: Vec<Box<dyn Visitor<Vec<Finding>>>>,
        root_abs_path: PathBuf,
    ) -> Self {
        Walker {
            artifact,
            source_map,
            visitors,
            root_abs_path,
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

            let mut ast: TypedAst = ast.to_typed();
            let source_unit = &ast.source_unit;
            let source_id = source_unit.id;

            // dedup same sources
            // TODO: is that bug from the ast ?
            if !ids.contains(&source_id) {
                ids.push(source_id);

                let abs_path = id.source.to_str().unwrap().to_string();

                let source_map_with_content = source_map
                    .get(&abs_path)
                    .unwrap_or(&(String::new(), Vec::new()))
                    .clone();

                let file_content = source_map_with_content.0;
                let lines_to_bytes = source_map_with_content.1;

                let path = PathBuf::from(&source_unit.absolute_path);

                let root = &self
                    .root_abs_path
                    .canonicalize()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();

                // the file may be outside the project. In that case, it's rather a lib.
                let name = path.strip_prefix(root).unwrap_or(&path).to_str().unwrap();

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
                        file_content.clone(),
                    );
                });
            }
        });

        // let mut findings: AllFindings = HashMap::new();

        // dedup same findings of the same module name, code, and src to avoid useless verbosity

        // Ok(findings)
        Ok(all_findings)
    }
}

pub fn visit_source<D>(
    source: &mut TypedAst,
    visitor: &mut Box<dyn Visitor<Vec<Finding>>>,
    lines_to_bytes: &[usize],
    info: Information,
    findings: &mut AllFindings,
    file_content: String,
) {
    source
        .clone()
        .visit(visitor.as_mut())
        .expect("ast traversal failed!");

    let file = info.name;

    let data = visitor.shared_data();

    data.iter().for_each(|finding| {
        let src = finding.src.as_ref().unwrap_or(&SourceLocation {
            start: Some(0),
            length: Some(0),
            index: Some(0),
        });

        let (position, content) = if let Some(start) = src.start {
            (
                get_position(start, lines_to_bytes),
                get_finding_content(file_content.clone(), start, src.length.unwrap_or_default()),
            )
        } else {
            // ((0, 0), String::new())
            ((0, 0), String::from("Error fetching content"))
        };

        let meta_finding = MetaFinding {
            finding: finding.clone(),
            meta: Meta {
                file: file.clone(),
                line: Some(position.0),
                position: Some(position.1),
                content,
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
