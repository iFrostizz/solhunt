use crate::build_visitor;

build_visitor! {
    BTreeMap::from([
        // https://github.com/code-423n4/2023-01-timeswap-findings/blob/main/data/0xSmartContract-G.md#g-03-avoid-using-state-variable-in-emit-130-gas
        (
            0,
            FindingKey {
                summary: "Using a state variable in an event emission wastes gas".to_string(),
                description: "A state variable should not be used in an event emission because it will load it from the storage. It should rather be loaded from the stack or the memory.".to_string(),
                severity: Severity::Gas,
            }
        ),
        // https://github.com/code-423n4/2023-01-timeswap-findings/blob/main/data/0xSmartContract-G.md#g-04-change-public-state-variable-visibility-to-private
        (
            1,
            FindingKey {
                summary: "Avoid using public for state variables".to_string(),
                description: "Public state variable are generating a getter function which costs more gas on deployment. Some variables may not need to require a getter function.".to_string(),
                severity: Severity::Gas,
            }
        ),
        // https://github.com/code-423n4/2023-01-timeswap-findings/blob/main/data/0xSmartContract-G.md#g-06-gas-savings-can-be-achieved-by-changing-the-model-for-assigning-value-to-the-structure-260-gas
        (
            2,
            FindingKey {
                summary: "Assign values to the struct directly".to_string(),
                description: "Assigning the values to the storage struct rather than one by one saves gas".to_string(),
                severity: Severity::Gas,
            }
        )
    ]),

    fn visit_source_unit(&mut self, source_unit: &mut SourceUnit) {
        source_unit.visit(self)?;

        self.events.clone().iter().for_each(|event| {
            let args = &event.event_call.arguments.clone();
            args.iter().for_each(|arg| {
                if let Expression::Identifier(identifier) = arg {
                    if self.state_variables.contains(&identifier.name) {
                        self.push_finding(0, Some(event.src.clone()));
                    }
                }
            });
        });

        Ok(())
    },

    fn visit_variable_declaration(&mut self, variable_declaration: &mut VariableDeclaration) {
        if variable_declaration.state_variable {
            self.state_variables.insert(variable_declaration.name.clone());

            if variable_declaration.visibility == Visibility::Public {
                self.push_finding(1, Some(variable_declaration.src.clone()));
            }
        }

        variable_declaration.visit(self)
    },

    fn visit_emit_statement(&mut self, emit_statement: &mut EmitStatement) {
        self.events.push(emit_statement.clone());

        emit_statement.visit(self)
    }
}

#[test]
fn state_var_in_event() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("StateVarEvent"),
        String::from(
            "pragma solidity 0.8.0;

contract StateVarEvent {
    event Mint(uint256);

    uint256 public supply;

    function wastesGas() public {
        emit Mint(supply); 
    }
}",
        ),
    )]);

    assert_eq!(lines_for_findings_with_code_module(&findings, "state", 0), vec![9]);
}

#[test]
fn stack_var_in_event() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("StateVarEvent"),
        String::from(
            "pragma solidity 0.8.0;

contract StateVarEvent {
    event Mint(uint256);

    function wastesGas() public {
        uint256 supply = 1_000_000;

        emit Mint(supply); 
    }
}",
        ),
    )]);

    assert!(!has_with_code(&findings, "state", 0));
}

#[test]
fn using_public_state() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("StateVarPublic"),
        String::from(
            "pragma solidity 0.8.0;

contract StateVarPublic {
    uint256 public supply;
}",
        ),
    )]);

    assert_eq!(lines_for_findings_with_code_module(&findings, "state", 1), vec![4]);
}

#[test]
fn state_struct_one_by_one() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("OneStruct"),
        String::from(
            "pragma solidity 0.8.0;

struct Parameters {
    address token0;
    address token1;
    uint112 reserve0;
    uint112 reserve1;
}

contract OneStruct {
    Parameters public parameters;

    function SetParams(
        address token0,
        address token1,
        uint112 reserve0,
        uint112 reserve1
    ) public {
        parameters.token0 = token0;
        parameters.token1 = token1;
        parameters.reserve0 = reserve0;
        parameters.reserve1 = reserve1;
    }
}",
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "state", 1),
        vec![19]
    );
}
