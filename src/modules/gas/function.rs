use crate::build_visitor;

build_visitor! {
    BTreeMap::from([
        // https://gist.github.com/hrkrshnn/ee8fabd532058307229d65dcd5836ddc#use-calldata-instead-of-memory-for-function-parameters
        (
            0,
            FindingKey {
                summary: "Use calldata instead of memory for function parameters".to_string(),
                description: "It is generally cheaper to load variables directly from calldata for external functions, rather than copying them to memory. Only use memory if the variable needs to be modified. ".to_string(),
                severity: Severity::Gas
            }
        )
    ]),

    fn visit_function_definition(&mut self, func: &mut FunctionDefinition) {
        if func.visibility == Visibility::External {
            func.parameters.parameters.iter().for_each(|param| {
                if param.storage_location == StorageLocation::Memory {
                    self.push_finding(0, Some(param.src.clone()));
                }
            });
        }

        Ok(())
    }
}

#[test]
fn memory_param() {
    let findings = compile_contract_and_get_findings(String::from(
        "pragma solidity 0.8.0;

contract Mem {
    function loop(uint[] memory arr) external pure returns (uint sum) {
        for (uint i = 0; i < arr.length; i++) {
            sum += arr[i];
        }
    }
}",
    ));

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "function", 0),
        vec![4]
    );
}

#[test]
fn calldata_param() {
    let findings = compile_contract_and_get_findings(String::from(
        "pragma solidity 0.8.0;

contract Mem {
    function loop(uint[] calldata arr) external pure returns (uint sum) {
        for (uint i = 0; i < arr.length; i++) {
            sum += arr[i];
        }
    }
}",
    ));

    assert!(!has_with_code(&findings, "function", 0));
}
