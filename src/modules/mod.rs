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
        use ethers_solc::artifacts::{visitor::{Visitor, VisitError, Visitable}, *, ast::*};
        #[allow(unused)]
        use $crate::{walker::{Finding, FindingMap, FindingKey, Severity, Inside, ModuleState}, loader::PushedFinding, solidity::{ProjectFile, compile_and_get_findings}, test::{lines_for_findings_with_code, has_with_code, has_with_module}};
        use ethers_solc::artifacts::ast::SourceLocation;
        #[allow(unused)]
        use semver::{Version, VersionReq};
        use std::{collections::BTreeMap};
        #[allow(unused)]
        use ethers_contract::BaseContract;
        #[allow(unused)]
        use ethers_core::abi::parse_abi;

        #[allow(dead_code)]
        pub struct DetectionModule {
            version: Option<Version>,
            // findings: Vec<Finding>,
            findings_map: FindingMap,
            pub function_definitions: Vec<FunctionDefinition>,
            pub function_calls: Vec<FunctionCall>,
            /// wether or not the visitor is inside a function
            pub inside: Inside,
            pub state_variables: Vec<String>,
            pub events: Vec<EmitStatement>,
            pub shared_data: ModuleState
        }

        /// populate the f_map on startup in order to specify the finding codes only
        impl Default for DetectionModule {
            fn default() -> Self {
                Self {
                    version: None,
                    // findings: Vec::new(),
                    findings_map: $map,
                    function_definitions: Vec::new(),
                    function_calls: Vec::new(),
                    state_variables: Vec::new(),
                    events: Vec::new(),
                    inside: Default::default(),
                    shared_data: ModuleState {
                        name: get_module_name(),
                        findings: Vec::new(),
                    },
                }
            }
        }

        trait FindingsPusher {
            fn new(findings_map: FindingMap) -> Self;
            fn push_finding(&mut self, code: usize, src: Option<SourceLocation>);
            fn push_findings(&mut self, f: Vec<PushedFinding>);
            fn p_finding(&mut self, code: usize, src: Option<SourceLocation>);
        }

        impl FindingsPusher for DetectionModule {
            fn new(findings_map: FindingMap) -> Self {
                Self {
                    findings_map,
                    ..Default::default()
                }
            }

            fn push_finding(&mut self, code: usize, src: Option<SourceLocation>) {
                self.p_finding(code, src);
            }

            fn push_findings(&mut self, findings: Vec<PushedFinding>) {
                findings.iter().for_each(|f| {
                    self.p_finding(f.code, f.src.clone());
                });
            }

            // TODO: allow having the same module names across folders
            fn p_finding(&mut self, code: usize, src: Option<SourceLocation>) {

                let name = get_module_name();

                let f_key = &self.findings_map.get(&code).expect("Unrecognized finding code");

                let finding = Finding {
                    name,
                    code,
                    summary: f_key.summary.clone(),
                    severity: f_key.severity.clone(),
                    description: f_key.description.clone(),
                    src
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
