// https://github.com/Picodes/4naly3er/blob/main/src/issues/GAS/addressZero.ts

use crate::build_visitor;

build_visitor! {
    BTreeMap::from([
        // https://medium.com/@kalexotsu/solidity-assembly-checking-if-an-address-is-0-efficiently-d2bfe071331
        (0,
            FindingKey {
                description: "Use assembly to check for `address(0)`, *Saves 6 gas per instance*".to_string(),
                summary: "address(0) check".to_string(),
                severity: Severity::Gas,
            }
        )
    ]),

    fn visit_binary_operation(&mut self, binary_operation: &mut BinaryOperation) {
        let lhs = &binary_operation.lhs;
        let rhs = &binary_operation.rhs;
        let operator = &binary_operation.operator;

        match operator {
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                self.push_findings(check_address_zero(lhs));
                self.push_findings(check_address_zero(rhs));
            },
            _ => ()
        }

        binary_operation.visit(self)
    }
}

fn check_address_zero(expr: &Expression) -> Vec<PushedFinding> {
    if let Expression::FunctionCall(func_call) = expr {
        if FunctionCallKind::TypeConversion == func_call.kind {
            if let Some(Expression::Literal(lit)) = func_call.arguments.get(0) {
                if lit.value == Some("0".to_owned()) {
                    return vec![PushedFinding {
                        src: Some(lit.src.clone()),
                        code: 0,
                    }];
                }
            }
        }
    }

    vec![]
}

#[cfg(test)]
mod tests {
    use ethers_core::utils::hex;
    use revm::primitives::{Bytes, B160, U256};

    use crate::interpreter::GasComparer;

    use super::*;

    // #[test]
    // fn address_zero_gas_optimization() {
    //     let abi = BaseContract::from(parse_abi(&["function saveMoney() external"]).unwrap());
    //     let encoded = abi.encode("saveMoney", ()).unwrap();
    //     let mut gas_compare = GasComparer::new(
    //         String::from(
    //             "pragma solidity 0.8.0;

    //             contract AddressZero {
    //                 event GasSaved();

    //                 function saveMoney() public {
    //                     if (msg.sender == address(0)) {
    //                         emit GasSaved();
    //                     }
    //                 }
    //             }",
    //         ),
    //         String::from(
    //             "pragma solidity 0.8.0;

    //             contract AddressZero {
    //                 event GasSaved();

    //                 function saveMoney() public {
    //                     bool zero;

    //                     assembly {
    //                         zero := iszero(caller())
    //                     }

    //                     if (zero) {
    //                         emit GasSaved();
    //                     }
    //                 }
    //             }",
    //         ),
    //         B160::default(),
    //         Bytes::from(hex::decode(hex::encode(&encoded)).unwrap()),
    //         U256::default(),
    //     );

    //     let (before, after) = gas_compare.run();

    //     println!("{before} {after}");
    // }

    #[test]
    fn finds_address_zero() {
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("AddressZero"),
            String::from(
                "pragma solidity 0.8.0;

                contract AddressZero {
                    event GasSaved();

                    function saveMoney() public {
                        if (msg.sender == address(0)) {
                            emit GasSaved();
                        }
                    }
                }",
            ),
        )]);

        assert_eq!(
            lines_for_findings_with_code_module(&findings, "address_zero", 0),
            vec![7]
        );
    }
}
