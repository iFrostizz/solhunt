// https://github.com/code-423n4/2022-12-tigris-findings/blob/main/data/JC-G.md#g02-state-variables-that-never-change-should-be-declared-immutable-or-constant

use crate::build_visitor;

build_visitor! {
    BTreeMap::from([
        (
            0,
            FindingKey {
                summary: "State variables that never change should be directly inlined in the bytecode".to_string(),
                description: "When state variables are guaranteed to never change, they should be inlined in the bytecode of the contract by declaring them as immutables or constants to avoid paying the upfront cost of SLOAD which are expensive, mainly when the slot is cold.".to_string(),
                severity: Severity::Gas,
            }
        )
    ]),

    fn visit_source_unit(&mut self, source_unit: &mut SourceUnit) {
        // get state variables and assignment
        // get all variables only assigned in the constructor or in the state directly
        source_unit.visit(self)?;
        // check if assignments are done on state variables out of constructor

        self.constructor_variables.clone().iter().for_each(|var| {
            if self.state_variables.contains(var) {
                let var_declaration = &self.state_name_to_var[var];
                self.push_finding(0, Some(var_declaration.src.clone()))
            }
        });

        Ok(())
    },

    fn visit_variable_declaration(&mut self, variable_declaration: &mut VariableDeclaration) {
        // push to the vec of state variable names
        if variable_declaration.state_variable {
            let name = &variable_declaration.name;
            self.state_variables.insert(name.clone());
            self.state_name_to_var.insert(name.clone(), variable_declaration.clone());
        }

        variable_declaration.visit(self)
    },

    fn visit_function_definition(&mut self, function_definition: &mut FunctionDefinition) {
        // check if we are in a constructor or not
        if function_definition.kind == Some(FunctionKind::Constructor) {
            self.inside.constructor = true;
        }

        function_definition.visit(self)?;

        self.inside.constructor = false;

        Ok(())
    },

    fn visit_assignment(&mut self, assignment: &mut Assignment) {
        // make a list of all state variables changed in the constructor.
        // remove them if they are changed in a function that is not the constructor.
        if let Expression::Identifier(identifier) = &assignment.lhs {
            let var_name = &identifier.name;

            if self.inside.constructor {
                self.constructor_variables.insert(var_name.clone());
            } else {
                self.constructor_variables.remove(var_name);
            }
        }

        assignment.visit(self)
    }
}

#[test]
fn not_changing() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("NotChange"),
        String::from(
            "pragma solidity 0.8.0;

contract NotChange {
    string public baseURI;

    constructor(string memory _baseURI) {
        baseURI = _baseURI;
    }
}",
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code(&findings, "immutable", 0),
        vec![4]
    );
}

#[test]
fn changing() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("Changes"),
        String::from(
            "pragma solidity 0.8.0;

contract Changes {
    string public baseURI;

    constructor(string memory _baseURI) {
        baseURI = _baseURI;
    }

    function setURI(string memory _baseURI) public {
        baseURI = _baseURI;
    }
}",
        ),
    )]);

    assert!(!has_with_code(&findings, "immutable", 0));
}
