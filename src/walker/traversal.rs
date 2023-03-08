// Takes a load of modules and walk through the full ast. Should be kind enough to tell bugs

use super::{Meta, MetaFinding, ModuleState};
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

        let mut visitor_len = HashMap::new();

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

                let (file_content, lines_to_bytes) = source_map
                    .clone()
                    .get(&abs_path)
                    // .unwrap_or_else(|| {
                    //     let msg = format!("source map not found for {}", abs_path);
                    //     tracing::warn!(target: "walker::traversal", msg);
                    //     &(String::new(), Vec::new())
                    // })
                    .unwrap_or(&(String::new(), Vec::new()))
                    .clone();

                let path = PathBuf::from(&source_unit.absolute_path);

                let root = &self
                    .root_abs_path
                    .canonicalize()
                    .unwrap_or_else(|_| panic!("failed to canonicalize {:#?}", self.root_abs_path))
                    .to_str()
                    .unwrap()
                    .to_string();

                // the file may be outside the project. In that case, it's rather a lib.
                // TODO: remove each root ancestors until that strip returns Ok
                let name = path.strip_prefix(root).unwrap_or(&path).to_str().unwrap();

                let info = Information {
                    name: name.to_string(),
                    version: id.version.clone(),
                };

                self.visitors.iter_mut().for_each(|visitor| {
                    visit_source::<ModuleState>(
                        &mut ast,
                        visitor,
                        &lines_to_bytes,
                        info.clone(),
                        &mut all_findings,
                        file_content.clone(),
                        &mut visitor_len,
                    );
                });
            }
        });

        Ok(all_findings)
    }
}

pub fn visit_source<D>(
    source: &mut TypedAst,
    visitor: &mut Box<dyn Visitor<ModuleState>>,
    lines_to_bytes: &[usize],
    info: Information,
    findings: &mut AllFindings,
    file_content: String,
    visitor_len: &mut HashMap<String, usize>,
) {
    source
        .clone()
        .visit(visitor.as_mut())
        .expect("ast traversal failed!");

    let file = info.name;

    let data = visitor.shared_data();

    let findings_data = &data.findings;
    let visitor_name = &data.name;

    let current_len = *visitor_len.get(visitor_name).unwrap_or(&0);

    findings_data
        .iter()
        .enumerate()
        .filter(|(i, _)| i >= &current_len)
        .for_each(|(i, finding)| {
            let src = finding.src.as_ref().unwrap_or(&SourceLocation {
                start: Some(0),
                length: Some(0),
                index: Some(0),
            });

            let (position, content) = if let Some(start) = src.start {
                (
                    get_position(start, lines_to_bytes),
                    get_finding_content(
                        file_content.clone(),
                        start,
                        src.length.unwrap_or_default(),
                    ),
                )
            } else {
                ((0, 0), String::from("Error fetching content"))
            };

            let meta_finding = MetaFinding {
                finding: finding.clone(),
                meta: Meta {
                    file: file.clone(),
                    line: Some(position.0),
                    width: Some(position.1),
                    content,
                },
            };

            // println!("{:#?}", meta_finding);

            // TODO: make a dedup data-structure. We don't wanna have the exact same finding on the same node anyway
            findings
                .entry(finding.name.clone())
                .and_modify(|f| f.push(meta_finding.clone()))
                .or_insert(vec![meta_finding.clone()]);

            visitor_len.insert(visitor_name.to_string(), i + 1);
        });
}
