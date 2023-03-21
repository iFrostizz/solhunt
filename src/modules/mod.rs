pub mod gas;
pub mod high;
pub mod info;
pub mod low;
pub mod medium;

pub mod oz;
pub mod uint256;

/// Build an implementation of a Visitor, without the boiler-plate
#[macro_export]
macro_rules! build_visitor {
    ($map:expr, $(fn $func_name:ident (&mut $self:ident, $($param:ident : $type:ty),*) $(-> $return_type:ty)* $body:block),*) => {
        // compiler complains for Visitable, but is actually needed.
        #[allow(unused)]
        use ethers_solc::artifacts::{visitor::{Visitor, VisitError, Visitable}, *, ast::{*, yul::*}};
        #[allow(unused)]
        use $crate::{walker::{Finding, FindingMap, FindingKey, Severity, Inside, ModuleState}, loader::PushedFinding, solidity::{ProjectFile, compile_and_get_findings}, test::*};
        use ethers_solc::artifacts::ast::SourceLocation;
        #[allow(unused)]
        use semver::{Version, VersionReq};
        #[allow(unused)]
        use std::collections::{BTreeMap, HashSet, HashMap};
        #[allow(unused)]
        use ethers_contract::BaseContract;
        #[allow(unused)]
        use ethers_core::abi::parse_abi;

        // TODO: these are valid across files, should clean the state if needed
        #[allow(dead_code)]
        pub struct DetectionModule {
            version: Option<Version>,
            // findings: Vec<Finding>,
            findings_map: FindingMap,
            pub function_definitions: Vec<FunctionDefinition>,
            pub function_calls: Vec<FunctionCall>,
            /// wether or not the visitor is inside a block
            pub inside: Inside,
            pub state_variables: HashSet<String>,
            pub assigned_variables: HashSet<String>,
            pub state_name_to_var: HashMap<String, VariableDeclaration>,
            /// variables assigned in the constructor or in the state only
            pub constructor_variables: HashSet<String>,
            pub events: Vec<EmitStatement>,
            pub shared_data: ModuleState,
            pub revert_reasons: HashMap<String, Vec<SourceLocation>>
        }

        /// populate the f_map on startup in order to specify the finding codes only
        impl Default for DetectionModule {
            fn default() -> Self {
                Self {
                    version: None,
                    findings_map: $map,
                    function_definitions: Vec::new(),
                    function_calls: Vec::new(),
                    state_variables: HashSet::new(),
                    state_name_to_var: HashMap::new(),
                    constructor_variables: HashSet::new(),
                    assigned_variables: HashSet::new(),
                    events: Vec::new(),
                    inside: Default::default(),
                    shared_data: ModuleState {
                        name: get_module_name(),
                        current_file: Default::default(),
                        findings: Vec::new(),
                        file_findings: HashMap::new(),
                    },
                    revert_reasons: HashMap::new()
                }
            }
        }

        trait FindingsPusher {
            fn new(findings_map: FindingMap) -> Self;
            fn push_finding(&mut self, code: usize, src: Option<SourceLocation>);
            fn push_finding_comment(&mut self, code: usize, src: Option<SourceLocation>, comment: String);
            fn push_findings(&mut self, f: Vec<PushedFinding>);
            fn p_finding(&mut self, code: usize, src: Option<SourceLocation>, comment: Option<String>);
        }

        impl FindingsPusher for DetectionModule {
            fn new(findings_map: FindingMap) -> Self {
                Self {
                    findings_map,
                    ..Default::default()
                }
            }

            fn push_finding(&mut self, code: usize, src: Option<SourceLocation>) {
                self.p_finding(code, src, None);
            }

            fn push_finding_comment(&mut self, code: usize, src: Option<SourceLocation>, comment: String) {
                self.p_finding(code, src, Some(comment));
            }

            fn push_findings(&mut self, findings: Vec<PushedFinding>) {
                findings.iter().for_each(|f| {
                    self.p_finding(f.code, f.src.clone(), None);
                });
            }

            fn p_finding(&mut self, code: usize, src: Option<SourceLocation>, comment: Option<String>) {
                let name = get_module_name();

                let f_key = &self.findings_map.get(&code).expect("Unrecognized finding code");

                let finding = Finding {
                    name,
                    code,
                    summary: f_key.summary.clone(),
                    severity: f_key.severity.clone(),
                    description: f_key.description.clone(),
                    src,
                    comment,
                    gas: Some(0)
                };

                self.shared_data.findings.push(finding);
            }
        }

        impl Visitor<ModuleState> for DetectionModule {
            fn shared_data(&mut self) -> &ModuleState {
                &self.shared_data
            }

            $(
                fn $func_name(&mut $self, $($param : $type),*) -> Result<(), VisitError> $body
            )*
        }

        fn get_module_name() -> String {
                let name = module_path!();
                name.rsplit_once(":").expect("Should call from modules").1.to_string()
        }
    }
}
