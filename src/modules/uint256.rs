// A silly module that finds all uint256

use crate::build_visitor;

build_visitor!(
    BTreeMap::from([(
        0,
        FindingKey {
            description: "We just found a uint256 yay!".to_string(),
            summary: "Dumb uint256".to_string(),
            severity: Severity::Informal
        }
    )]),
    fn visit_variable_declaration(&mut self, var: &mut VariableDeclaration) {
        if let Some(type_id) = &var.type_descriptions.type_identifier {
            if type_id == "t_uint256" {
                self.push_finding(0, Some(var.src.clone()));
            }
        }

        var.visit(self)
    }
);

// #[cfg(test)]
// mod test {
//     use crate::{
//         solidity::ProjectFile,
//         test::{compile_and_get_findings, lines_for_findings_with_code},
//     };

//     #[test]
//     fn can_find_dummy_uint256() {
//         let findings = compile_and_get_findings(vec![ProjectFile::Contract(
//             String::from("DummyUint256"),
//             String::from(
//                 "pragma solidity 0.8.0;

//             contract DummyUint256 {
//                 uint256 unint;
//             }
//             ",
//             ),
//         )]);

//         assert_eq!(
//             lines_for_findings_with_code(&findings, "uint256", 0),
//             vec![4]
//         );
//     }
// }
