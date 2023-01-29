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
    ($(fn $func_name:ident ($($opt:expr),*) $(-> $return_type:ty)* $block:block)*) => {
        #[derive(Default)]
    pub struct DetectionModule {
        findings: Vec<Finding>
    }

impl Visitor<Vec<Finding>> for DetectionModule {
            fn shared_data(&mut self) -> &Vec<Finding> {
                &self.findings
            }

        [$($func_name),*]

        }
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
