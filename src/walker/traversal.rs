// Takes a load of modules and walk through the full ast. Should be kind enough to tell bugs

use super::{Meta, MetaFinding, ModuleState};
use crate::{
    cmd::bars::get_bar,
    loader::Information,
    solidity::{get_finding_content, get_finding_content_arrow, get_position},
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
    cell::RefCell,
    collections::{btree_map::BTreeMap, HashMap, HashSet},
    ops::DerefMut,
    path::PathBuf,
    rc::Rc,
};

pub struct Walker {
    artifacts: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
    source_map: BTreeMap<String, (String, Vec<usize>)>,
    visitors: Vec<Rc<RefCell<dyn Visitor<ModuleState>>>>,
    display_bar: bool,
}

impl Walker {
    pub fn new(
        artifacts: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
        source_map: BTreeMap<String, (String, Vec<usize>)>,
        visitors: Vec<Rc<RefCell<dyn Visitor<ModuleState>>>>,
    ) -> Self {
        Walker {
            artifacts,
            source_map,
            visitors,
            display_bar: false,
        }
    }

    pub fn with_bar(mut self, flag: bool) -> Self {
        self.display_bar = flag;
        self
    }

    // For analyzing a syntax tree, we need an AST "walker" — an object to facilitate the traversal of the tree.
    // The ast module offers two walkers:
    // - ast.NodeVisitor (doesn't allow modification to the input tree)
    // - ast.NodeTransformer (allows modification)
    pub fn traverse(&mut self) -> eyre::Result<AllFindings> {
        let mut all_findings: AllFindings = HashMap::new();

        let mut ids: Vec<usize> = Vec::new();
        let source_map = &self.source_map.clone();

        let sources: Vec<_> = self
            .artifacts
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

                let abs_path = &id.source;

                // the file may be outside the project. In that case, it's rather a lib.
                // TODO: remove each root ancestors until that strip returns Ok
                let name = &source_unit.absolute_path;

                let info = Information {
                    name: name.to_string(),
                    version: id.version.clone(),
                };

                let ret = if ids.contains(&source_id) {
                    None
                } else {
                    Some((ast, info, abs_path))
                };

                ids.push(source_id);

                ret
            })
            .collect();

        let bar = if self.display_bar {
            Some(get_bar(self.visitors.len() as u64, Default::default()))
        } else {
            None
        };

        // TODO: parallel this
        self.visitors.iter().try_for_each(|visitor| {
            {
                let mut vis = visitor.borrow_mut();
                let data = vis.shared_data();

                if self.display_bar {
                    let b = bar.as_ref().unwrap();
                    b.set_message(data.name.clone());
                    b.inc(1);
                }
            }

            visit_sources::<ModuleState>(sources.clone(), visitor, source_map, &mut all_findings)
        })?;

        Ok(all_findings)
    }
}

pub fn visit_sources<D>(
    full_sources: Vec<(TypedAst, Information, &PathBuf)>,
    visitor: &Rc<RefCell<dyn Visitor<ModuleState>>>,
    source_map: &BTreeMap<String, (String, Vec<usize>)>,
    findings: &mut AllFindings,
) -> eyre::Result<()> {
    let mut last_id = 0usize;
    let mut visitor = visitor.borrow_mut();

    for (mut source, info, abs_path) in full_sources.into_iter() {
        source.visit(visitor.deref_mut())?;

        let data = visitor.shared_data();
        let findings_data = &data.findings.to_vec();

        let source_findings = &findings_data[last_id..].to_vec();

        source_findings.iter().for_each(|finding| {
            let (position, content) = if let Some(src) = &finding.src {
                if let Some(start) = src.start {
                    if let Some((file_content, lines_to_bytes)) =
                        source_map.get(abs_path.to_str().unwrap())
                    {
                        (
                            get_position(start, lines_to_bytes),
                            get_finding_content_arrow(
                                file_content.to_string(),
                                start,
                                src.length.unwrap_or_default(),
                            ),
                        )
                    } else {
                        ((0, 0), String::from("Path not indexed in source map"))
                    }
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
                .and_modify(|f| {
                    f.insert(meta_finding.clone());
                })
                .or_insert(HashSet::from([meta_finding.clone()]));
        });

        last_id = findings_data.len();
    }

    Ok(())
}
