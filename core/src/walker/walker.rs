// Takes a load of modules and walk through the full ast. Should be kind enough to tell bugs

use ethers_solc::{artifacts::ast::Node, ArtifactId, ConfigurableContractArtifact};
use std::collections::btree_map::BTreeMap;
use std::collections::HashMap;
use std::{fs::File, io::BufReader};

use crate::{
    loader::Information,
    solidity::utils::{get_file_lines, get_line_position},
    walker::{AllFindings, Findings, Meta, MetaFinding},
};
use crate::loader::{DynModule, Loader};

pub struct Walker
{
    artifact: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
    loader: Loader,
}

impl Walker
{
    pub fn new(
        artifact: BTreeMap<ArtifactId, ConfigurableContractArtifact>,
        loader: Loader,
    ) -> Self {
        Walker { artifact, loader }
    }

    /*
    For analyzing a syntax tree, we need an AST "walker" — an object to facilitate the traversal of the tree.
        The ast module offers two walkers:
            - ast.NodeVisitor (doesn't allow modification to the input tree)
            - ast.NodeTransformer (allows modification)
     */
    pub fn traverse(&mut self) -> eyre::Result<AllFindings> {
        // Should output a friendly stuff
        let mut all_findings: AllFindings = HashMap::new();

        for (id, art) in &self.artifact {
            // dbg!(&id.version, &id.name, &id.identifier()); Careful, pragma version is the COMPILED version. Should parse
            // probably fine to use the compiled version, if major change then it wouldn't compile.
            let unique_id = format!("{} {}", id.name, id.identifier());
            let ast = art
                .ast
                .as_ref()
                .expect(format!("no ast found for {}", unique_id).as_str());

            let lines_to_bytes = match File::open(ast.absolute_path.clone()) {
                // only use if file is found
                Ok(file) => {
                    let file = BufReader::new(file);
                    get_file_lines(file).expect("failed to parse lines")
                }
                Err(_) => Vec::new(),
            };
            let lines_to_bytes = &lines_to_bytes;

            let nodes = &ast.nodes;

            let name = &ast.absolute_path;

            let info = Information {
                name: name.to_string(),
                version: id.version.clone(),
            };

            self.loader.0.iter().for_each(|module| {
                all_findings.entry(module.name.clone()).or_default();
                let mut findings: &mut Findings = &mut Vec::new();
                self.visit_nodes(module, nodes, lines_to_bytes, info.clone(), &mut findings);
                all_findings
                    .entry(module.name.clone())
                    .and_modify(|f| f.append(findings));
            });
        }

        Ok(all_findings)
    }

    pub fn visit_nodes(
        &self,
        module: &DynModule,
        nodes: &Vec<Node>,
        lines_to_bytes: &Vec<usize>,
        info: Information,
        findings: &mut Findings,
    ) {
        nodes.into_iter().for_each(|node| {
            if let Some(finding) = module.process(&node, &info) {
                let meta = Meta {
                    file: info.name.clone(),
                    src: get_line_position(&node.src, lines_to_bytes),
                };
                let meta_finding = MetaFinding { finding, meta };

                findings.push(meta_finding);
            }
            /*match node.node_type {
                NodeType::PragmaDirective => {
                    let directive = node.other.get("literals").unwrap().clone();
                    version_from_literals(directive);
                    // dbg!(&directive);
                }
                NodeType::ContractDefinition => {
                    let name = node.other.get("canonicalName").unwrap().clone();
                    let kind = node.other.get("contractKind").unwrap().clone();
                    println!("Contract name: {} kind: {}", name, kind);
                }
                NodeType::VariableDeclaration => {
                    // dbg!(&node);
                    let type_name = node.other.get("typeName").unwrap().clone();
                    // dbg!(type_name);
                    // dbg!(&type_name.get("valueType").unwrap().get("nodeType").unwrap());
                    /*let inner_name = match type_name.get("valueType").unwrap().get("nodeType").unwrap() {
                        NodeType::ElementaryTypeName => {
                            type_name.get("name").unwrap()
                        }
                        NodeType::Other(val) => {
                            panic!("{} not implemented", val);
                        }
                    };

                    let name = node.other.get("name").unwrap().clone();
                    println!("Variable: {} {}", inner_name, name);*/
                }
                NodeType::FunctionDefinition => {
                    let kind = node.other.get("kind").unwrap().clone();
                    let name = node.other.get("name").unwrap().clone();
                    let visibility = node.other.get("visibility").unwrap().clone();
                    println!("Function: kind {} name {}() visibility {}", kind, name, visibility);
                }
                _ => ()
            }*/
            // dbg!(&node.id);*/
            let inner_nodes = &node.nodes;

            self.visit_nodes(module, inner_nodes, lines_to_bytes, info.clone(), findings);
        });
    }
}
