pub mod assembly;
pub mod calls;
pub mod chainlink;
pub mod erc20;
pub mod loader;
pub mod overflow;
pub mod oz;
pub mod style;
pub mod uint256;

// TODO: add the map in the macro arguments
/// Build an implementation of a Visitor, without the boiler-plate
#[macro_export]
macro_rules! build_visitor {
    ($(fn $func_name:ident (&mut $self:ident, $($param:ident : $type:ty),*) $(-> $return_type:ty)* $body:block)*) => {
        // ($(fn $func_name:ident ($($opt:expr),*) $(-> $return_type:ty)* $block:block)*) => {
        use ethers_solc::artifacts::visitor::{Visitor, VisitError, Visitable};
        use ethers_solc::artifacts::*;
        use $crate::walker::{Finding, FindingMap};
        use ethers_solc::artifacts::ast::SourceLocation;

        // TODO: populate the f_map on startup
        // Can either make a hook in the visitor or in the Default implementation
        #[derive(Default)]
        pub struct DetectionModule {
            findings: Vec<Finding>,
            findings_map: FindingMap
        }

        trait FindingsPusher {
            fn new(findings_map: FindingMap) -> Self;
            fn push_finding(&mut self, src: Option<SourceLocation>, code: u32);        }

        impl FindingsPusher for DetectionModule {
            fn new(findings_map: FindingMap) -> Self {
                Self {
                    findings: Vec::new(),
                    findings_map
                }

            }

            fn push_finding(&mut self, src: Option<SourceLocation>, code: u32) {
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

            // TODO: rework the findings pushing with a map of finding code to description and severity
            // fn push_finding(&mut self, src: Option<SourceLocation>, code: u32) {

            //     let name = module_path!();
            //     let name = name.rsplit_once(':').expect("failed to split name from odules").[1];

            //     let finding = Finding {
            //         name,
            //         description,
            //         severity,
            //         src,
            //         code
            //     };

            //     self.findings.push(finding);
            // }

            // [$($func_name),*]

            $(
                fn $func_name(&mut $self, $($param : $type),*) -> Result<(), VisitError> $body
                )*
        }
    };
}

#[macro_export]
macro_rules! get_path {
    () => {
        println!("{}", module_path!());
    };
}

// macro_rules! build_visitor {
//     ($functions:tt) => {
//         #[derive(Default)]
//         pub struct DetectionModule {
//             findings: Vec<Finding>,
//         }

//         impl Visitor<Vec<Finding>> for DetectionModule {
//             fn shared_data(&mut self) -> &Vec<Finding> {
//                 &self.findings
//             }

//             $functions
//         }
//     };
// }
