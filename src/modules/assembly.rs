// Module that finds for external and dangerous calls

use crate::{
    loader::{DynModule, Module},
    walker::{Finding, Severity},
};
use ethers_solc::artifacts::{
    ast::{ContractDefinitionPart, SourceUnitPart},
    yul::{YulExpression, YulStatement},
    Block, Statement,
};

pub fn get_module() -> DynModule {
    Module::new(
        "assembly",
        Box::new(move |source, _info| {
            let mut findings: Vec<Finding> = Vec::new();

            if let SourceUnitPart::ContractDefinition(def) = source {
                def.nodes.iter().for_each(|node| {
                    if let ContractDefinitionPart::FunctionDefinition(func) = node {
                        if let Some(body) = &func.body {
                            findings.append(&mut parse_body(body));
                        }
                    }
                });
            }

            findings
        }),
    )
}

fn parse_body(body: &Block) -> Vec<Finding> {
    let mut findings = Vec::new();

    body.statements
        .iter()
        .for_each(|stat| findings.append(&mut check_for_assembly(stat)));

    findings
}

fn check_for_assembly(stat: &Statement) -> Vec<Finding> {
    let mut findings = Vec::new();

    if let Statement::InlineAssembly(in_ass) = stat {
        findings.push(Finding {
            name: "assembly".to_owned(),
            description: "usage of inline assembly, take extra care here".to_owned(),
            severity: Severity::Informal,
            src: Some(in_ass.src.clone()),
            code: 0,
        });

        in_ass
            .ast
            .statements
            .iter()
            .for_each(|s| findings.append(&mut recurse_assembly_statements(s)));
    }

    findings
}

fn recurse_assembly_statements(stat: &YulStatement) -> Vec<Finding> {
    let mut findings = Vec::new();

    if let YulStatement::YulAssignment(yul_ass) = stat {
        if let YulExpression::YulFunctionCall(function_call) = &yul_ass.value {
            let func_name = &function_call.function_name;

            if func_name.name == "extcodesize" {
                findings.push(Finding {
                    name: "assembly".to_owned(),
                    description: "using extcodesize. Can be an issue if determining if EOA."
                        .to_owned(),
                    severity: Severity::Medium,
                    src: Some(func_name.src.clone()),
                    code: 1,
                });
            }
        }
    } else if let YulStatement::YulForLoop(for_loop) = stat {
        for_loop
            .body
            .statements
            .iter()
            .for_each(|s| findings.append(&mut recurse_assembly_statements(&s)));
    }

    findings
}

#[cfg(test)]
mod tests {
    use crate::{
        solidity::ProjectFile,
        test::{compile_and_get_findings, has_with_code, lines_for_findings_with_code},
    };

    #[test]
    fn with_extcodesize() {
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("ExtCodeSize"),
            String::from(
                "pragma solidity ^0.8.0;

contract ExtCodeSize {
    function make(address to) public {
        uint256 size;
            
        assembly {
            size := extcodesize(to)
        }
    }
}",
            ),
        )]);

        assert_eq!(
            lines_for_findings_with_code(&findings, "assembly", 0), // usage of assembly
            vec![7]
        );

        assert_eq!(
            lines_for_findings_with_code(&findings, "assembly", 1), // extcodesize
            vec![8]
        );
    }

    #[test]
    fn without_extcodesize() {
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("WithoutExtCodeSize"),
            String::from(
                "pragma solidity ^0.8.0;

contract WithoutExtCodeSize {
    function make(address to) public {
        uint256 bal;
            
        assembly {
            bal := balance(to)
        }
    }
}",
            ),
        )]);

        assert_eq!(
            lines_for_findings_with_code(&findings, "assembly", 0), // usage of assembly
            vec![7]
        );

        assert!(!has_with_code(&findings, "assembly", 1)); // extcodesize);
    }

    #[test]
    fn nested_extcodesize() {
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("NestedExtCodeSize"),
            String::from(
                "pragma solidity ^0.8.0;

contract NestedExtCodeSize {
    function make(address to) public {
        uint256 size;
            
        assembly {
            for { let i:= 0 } lt(i, 10) { i := add(i, 1) } {
                size := extcodesize(to)
            }
        }
    }
}",
            ),
        )]);

        assert_eq!(
            lines_for_findings_with_code(&findings, "assembly", 0), // usage of assembly
            vec![7]
        );

        assert_eq!(
            lines_for_findings_with_code(&findings, "assembly", 1), // extcodesize
            vec![9]
        );
    }
}
