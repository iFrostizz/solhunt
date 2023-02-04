pub mod address_zero;
pub mod assembly;
pub mod calls;
pub mod centralization;
pub mod chainlink;
pub mod erc20;
pub mod overflow;
pub mod oz;
pub mod style;
pub mod uint256;

/// Build an implementation of a Visitor, without the boiler-plate
#[macro_export]
macro_rules! build_visitor {
    ($map:expr, $(fn $func_name:ident (&mut $self:ident, $($param:ident : $type:ty),*) $(-> $return_type:ty)* $body:block),*) => {
        // compiler complains for Visitable, but is actually needed.
        #[allow(unused)]
        use ethers_solc::artifacts::{visitor::{Visitor, VisitError, Visitable}, *, ast::*};
        #[allow(unused)]
        use $crate::{walker::{Finding, FindingMap, FindingKey, Severity, Inside}, loader::PushedFinding, solidity::ProjectFile, test::{compile_and_get_findings, lines_for_findings_with_code}};
        use ethers_solc::artifacts::ast::SourceLocation;
        #[allow(unused)]
        use semver::{Version, VersionReq};
        use std::{collections::BTreeMap};

        #[allow(dead_code)]
        pub struct DetectionModule {
            version: Option<VersionReq>,
            findings: Vec<Finding>,
            findings_map: FindingMap,
            /// wether or not the visitor is inside a function
            pub inside: Inside,
        }

        /// populate the f_map on startup in order to specify the finding codes only
        impl Default for DetectionModule {
            fn default() -> Self {
                Self {
                    findings: Vec::new(),
                    findings_map: $map,
                    version: None,
                    inside: Default::default()
                }
            }
        }

        trait FindingsPusher {
            fn new(findings_map: FindingMap) -> Self;
            fn push_finding(&mut self, src: Option<SourceLocation>, code: u32);
            fn push_findings(&mut self, f: Vec<PushedFinding>);
            fn p_finding(&mut self, src: Option<SourceLocation>, code: u32);
        }

        impl FindingsPusher for DetectionModule {
            fn new(findings_map: FindingMap) -> Self {
                Self {
                    findings: Vec::new(),
                    findings_map,
                    version: None,
                    inside: Default::default()
                }
            }

            fn push_finding(&mut self, src: Option<SourceLocation>, code: u32) {
                self.p_finding(src, code);
            }

            fn push_findings(&mut self, findings: Vec<PushedFinding>) {
                findings.iter().for_each(|f| {
                    self.p_finding(f.src.clone(), f.code);
                });
            }

            fn p_finding(&mut self, src: Option<SourceLocation>, code: u32) {
                let name = module_path!();
                let name = name.rsplit_once(":").expect("Should call from modules").1.to_string();

                let f_key = &self.findings_map.get(&code).expect("Unrecognized finding code");

                let finding = Finding {
                    name,
                    code,
                    severity: f_key.severity.clone(),
                    description: f_key.description.clone(),
                    src
                };

                self.findings.push(finding);
            }
        }

        impl Visitor<Vec<Finding>> for DetectionModule {
            fn shared_data(&mut self) -> &Vec<Finding> {
                &self.findings
            }

            $(
                fn $func_name(&mut $self, $($param : $type),*) -> Result<(), VisitError> $body
            )*
        }
    }
}
