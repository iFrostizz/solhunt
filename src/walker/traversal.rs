// Takes a load of modules and walk through the full ast. Should be kind enough to tell bugs

use super::{Meta, MetaFinding, ModuleState};
use crate::{
    loader::Information,
    solidity::{get_finding_content, get_position},
    walker::AllFindings,
};
use ethers_solc::{
    artifacts::{
        ast::lowfidelity::Ast,
        lowfidelity::TypedAst,
        visitor::{Visitable, Visitor},
    },
    ArtifactId, ConfigurableContractArtifact,
};
use std::{
    collections::{btree_map::BTreeMap, HashMap},
    path::PathBuf,
};

pub struct Walker {
    artifact: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
    source_map: BTreeMap<String, (String, Vec<usize>)>,
    visitors: Vec<Box<dyn Visitor<ModuleState>>>,
    root_abs_path: PathBuf,
}

impl Walker {
    pub fn new(
        artifact: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
        source_map: BTreeMap<String, (String, Vec<usize>)>,
        visitors: Vec<Box<dyn Visitor<ModuleState>>>,
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

        // let mut visitor_len = HashMap::new();

        let ids: Vec<usize> = Vec::new();
        let source_map = &self.source_map.clone();

        let sources: Vec<_> = self
            .artifact
            .iter()
            .filter_map(|(id, art)| {
                let unique_id = id.identifier();

                let ast: Ast = art
                    .ast
                    .as_ref()
                    .unwrap_or_else(|| panic!("no ast found for {unique_id}"))
                    .clone();

                let ast: TypedAst = ast.to_typed();
                let source_unit = &ast.source_unit;
                let source_id = source_unit.id;

                let abs_path = id.source.to_str().unwrap().to_string();

                let root = &self
                    .root_abs_path
                    .canonicalize()
                    .unwrap_or_else(|_| panic!("failed to canonicalize {:#?}", self.root_abs_path))
                    .to_str()
                    .unwrap()
                    .to_string();

                let path = PathBuf::from(&source_unit.absolute_path);

                // the file may be outside the project. In that case, it's rather a lib.
                // TODO: remove each root ancestors until that strip returns Ok
                let name = path.strip_prefix(root).unwrap_or(&path).to_str().unwrap();

                let info = Information {
                    name: name.to_string(),
                    version: id.version.clone(),
                };

                if ids.contains(&source_id) {
                    None
                } else {
                    Some((ast, info, abs_path))
                }
            })
            .collect();

        self.visitors.iter_mut().for_each(|visitor| {
            visit_sources::<ModuleState>(sources.clone(), visitor, source_map, &mut all_findings)
        });

        Ok(all_findings)
    }
}

pub fn visit_sources<D>(
    full_sources: Vec<(TypedAst, Information, String)>,
    visitor: &mut Box<dyn Visitor<ModuleState>>,
    source_map: &BTreeMap<String, (String, Vec<usize>)>,
    findings: &mut AllFindings,
) {
    let mut last_id = 0usize;

    full_sources
        .into_iter()
        .for_each(|(mut source, info, abs_path)| {
            let (file_content, lines_to_bytes) = source_map
                .clone()
                .get(&abs_path)
                .unwrap_or(&(String::new(), Vec::new()))
                .clone();

            source
                .visit(visitor.as_mut())
                .expect("ast traversal failed!");

            let data = visitor.shared_data();
            let findings_data = &data.findings.to_vec();

            let source_findings = &findings_data[last_id..].to_vec();

            source_findings.iter().for_each(|finding| {
                let (position, content) = if let Some(src) = &finding.src {
                    if let Some(start) = src.start {
                        (
                            get_position(start, &lines_to_bytes),
                            get_finding_content(
                                file_content.clone(),
                                start,
                                src.length.unwrap_or_default(),
                            ),
                        )
                    } else {
                        ((0, 0), String::from("No source map for this file"))
                    }
                } else {
                    ((0, 0), String::from("Error fetching content"))
                };

                let meta_finding = MetaFinding {
                    finding: finding.clone(),
                    meta: Meta {
                        file: info.name.clone(),
                        line: Some(position.0),
                        width: Some(position.1),
                        content,
                    },
                };

                assert_eq!(finding.name.clone(), data.name.to_string());
                findings
                    .entry(finding.name.clone())
                    .and_modify(|f| f.push(meta_finding.clone()))
                    .or_insert(vec![meta_finding.clone()]);
            });

            last_id = findings_data.len();
        });
}
