pub mod assembly;
pub mod calls;
pub mod chainlink;
pub mod erc20;
pub mod loader;
pub mod overflow;
pub mod oz;
pub mod style;
pub mod uint256;

// TODO: this does not work
// Food for thoughts:
// https://stackoverflow.com/questions/39986539/how-can-i-use-a-macro-to-create-an-array-of-function-names-starting-from-a-colle
// https://users.rust-lang.org/t/macro-for-option-arguments/70726/2
#[macro_export]
macro_rules! build_visitor {
    ($(fn $func_name:ident (&mut $self:ident, $($param:ident : $type:ty),*) $(-> $return_type:ty)* $body:block)*) => {
        // ($(fn $func_name:ident ($($opt:expr),*) $(-> $return_type:ty)* $block:block)*) => {
        use ethers_solc::artifacts::visitor::{Visitor, VisitError, Visitable};
        use ethers_solc::artifacts::*;
        use $crate::walker::Finding;
        // use ethers_solc::artifacts::ast::SourceLocation;

        // TODO: populate the f_map on startup
        // Can either make a hook in the visitor or in the Default implementation
        #[derive(Default)]
        pub struct DetectionModule {
            findings: Vec<Finding>,
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
