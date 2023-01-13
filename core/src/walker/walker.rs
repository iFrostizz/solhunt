// Takes a load of modules and walk through the full ast. Should be kind enough to tell bugs

use ethers_solc::{
    artifacts::ast::{lowfidelity::Ast, SourceUnit, SourceUnitPart},
    ArtifactId, ConfigurableContractArtifact,
};
use std::collections::HashMap;
use std::{collections::btree_map::BTreeMap, path::PathBuf};
// use std::{fs::File, io::BufReader};

use crate::loader::{DynModule, Loader};
use crate::{
    loader::Information,
    solidity::utils::get_line_position,
    walker::{AllFindings, Findings, Meta, MetaFinding},
};

pub struct Walker {
    artifact: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
    loader: Loader,
    source_map: BTreeMap<String, Vec<usize>>,
}

impl Walker {
    pub fn new(
        artifact: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
        loader: Loader,
        source_map: BTreeMap<String, Vec<usize>>,
    ) -> Self {
        Walker {
            artifact,
            loader,
            source_map,
        }
    }

    /*
    For analyzing a syntax tree, we need an AST "walker" — an object to facilitate the traversal of the tree.
        The ast module offers two walkers:
            - ast.NodeVisitor (doesn't allow modification to the input tree)
            - ast.NodeTransformer (allows modification)
     */
    pub fn traverse(&mut self) -> eyre::Result<AllFindings> {
        let mut all_findings: AllFindings = HashMap::new();

        let mut ids: Vec<usize> = Vec::new();

        for (id, art) in &self.artifact {
            // dbg!(&id.version, &id.name, &id.identifier()); Careful, pragma version is the COMPILED version. Should parse
            // probably fine to use the compiled version, if major change then it wouldn't compile.
            let unique_id = format!("{} {}", id.name, id.identifier());
            // dbg!(&unique_id);
            dbg!(id.identifier());

            /*art.generated_sources.iter().for_each(|gc| {
                println!("{:#?}", &gc.contents);
            });*/
            // println!("{:#?}", art.generated_sources);

            let ast: Ast = art
                .ast
                .as_ref()
                .unwrap_or_else(|| panic!("no ast found for {}", unique_id))
                .clone();

            let ast: &SourceUnit = &ast.to_typed();

            // dedup same sources
            // TODO: is that bug from the ast ?
            if !ids.contains(&ast.id) {
                ids.push(ast.id);

                // dbg!(&ast);

                /*let abs_path = &ast.absolute_path.clone();
                let file = File::open(abs_path)
                    .unwrap_or_else(|_| panic!("failed to open file at {}", abs_path));
                let file = BufReader::new(file);
                let lines_to_bytes = get_file_lines(file).expect("failed to parse lines");*/
                // let lines_to_bytes = [];
                let lines_to_bytes = &self.source_map.get(&id.identifier()).unwrap()/*.unwrap_or(&Vec::new())*/;
                // dbg!(&lines_to_bytes);

                let nodes = &ast.nodes;

                // let name = &ast.absolute_path;
                let path = PathBuf::from(&ast.absolute_path);
                let name = path.file_name().unwrap();
                let name = name.to_os_string().into_string().unwrap();
                let name = name.strip_suffix(".sol").unwrap();

                let info = Information {
                    name: name.to_string(),
                    version: id.version.clone(),
                };

                self.loader.0.iter().for_each(|module| {
                    all_findings.entry(module.name.clone()).or_default();
                    let findings: &mut Findings = &mut Vec::new();
                    self.visit_source(module, nodes, lines_to_bytes, info.clone(), findings);
                    all_findings
                        .entry(module.name.clone())
                        .and_modify(|f| f.append(findings));
                });
            }
        }

        Ok(all_findings)
    }

    pub fn visit_source(
        &self,
        module: &DynModule,
        sources: &[SourceUnitPart],
        lines_to_bytes: &[usize],
        info: Information,
        findings: &mut Findings,
    ) {
        sources.iter().for_each(|source| {
            let mod_findings = module.process_source(source, &info);

            let file = info.name.clone();

            let mut meta_findings: Findings = mod_findings
                .into_iter()
                .map(|finding| MetaFinding {
                    finding: finding.clone(),
                    meta: Meta {
                        file: file.clone(),
                        line: finding
                            .src
                            .map(|src| get_line_position(&src, lines_to_bytes) as u32),
                    },
                })
                .collect();

            findings.append(&mut meta_findings);
        });
    }
}
