use crate::build_visitor;

// TODO: https://github.com/code-423n4/2023-03-neotokyo-findings/blob/main/data/JCN-G.md#avoid-emitting-constants
// TODO: make it work with inherited contracts as well
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
                summary: "Avoid using public for immutable/constant state variables".to_string(),
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
        ),
        (
            3,
            FindingKey {
                summary: "Avoid using compound operators with state variables".to_string(),
                description: "+= and -= are more expensive than = + and = -".to_string(),
                severity: Severity::Gas
            }
        ),
        // TODO: avoid assigning state boolean, can be quantified more easily
        (
            4,
            FindingKey {
                summary: "Avoid using state booleans".to_string(),
                description: "more expensive than using uint256(1) and uint256(2)".to_string(),
                severity: Severity::Gas
            }
        ),
        (
            5,
            FindingKey {
                summary: "State variables that never change should be directly inlined in the bytecode".to_string(),
                description: "When state variables are guaranteed to never change, they should be inlined in the bytecode of the contract by declaring them as immutables or constants to avoid paying the upfront cost of SLOAD which are expensive, mainly when the slot is cold.".to_string(),
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

        self.state_variables.clone().iter().for_each(|var| {
            let var_d = &self.state_name_to_var[var];

            if let Some(td) = &var_d.type_descriptions.type_string {
                if !(
                    td.starts_with("mapping")
                    || td.starts_with("string")
                    || td.starts_with("bytes"))
                    && (self.constructor_variables.contains(var)
                        || !self.assigned_variables.contains(var))
                    && var_d.mutability() == &Mutability::Mutable {
                    self.push_finding(5, Some(var_d.src.clone()));
                }
            }
        });

        self.constructor_variables.clear();
        self.state_variables.clear();
        self.state_name_to_var.clear();
        self.assigned_variables.clear();

        Ok(())
    },

    fn visit_variable_declaration(&mut self, var: &mut VariableDeclaration) {
        if var.state_variable {
            let name = &var.name;
            self.state_variables.insert(name.clone());
            self.state_name_to_var.insert(name.clone(), var.clone());

            if var.visibility == Visibility::Public && var.mutability() != &Mutability::Mutable {
                self.push_finding(1, Some(var.src.clone()));
            }

            if var.type_descriptions.type_string == Some(String::from("bool")) {
                self.push_finding(4, Some(var.src.clone()));
            }
        }

        var.visit(self)
    },

    fn visit_emit_statement(&mut self, emit_statement: &mut EmitStatement) {
        self.events.push(emit_statement.clone());

        emit_statement.visit(self)
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
        if let Expression::Identifier(identifier) = &assignment.lhs {
            let var_name = &identifier.name;
            self.assigned_variables.insert(var_name.to_owned());

            if self.inside.constructor {
                self.constructor_variables.insert(var_name.clone());
            } else {
                self.constructor_variables.remove(var_name);
            }
        }

        match assignment.operator {
            AssignmentOperator::AddAssign | AssignmentOperator::SubAssign => {
                if let Expression::Identifier(id) = &assignment.lhs {
                    if self.state_variables.contains(&id.name) {
                        self.push_finding(3, Some(assignment.src.clone()));
                    }
                }
            }
            _ => ()
        }

        assignment.visit(self)
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

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "state", 0),
        vec![9]
    );
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

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "state", 5),
        vec![4]
    );
}

#[test]
fn immut_public() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("Immut"),
        String::from(
            "pragma solidity 0.8.0;

contract Immut {
    uint256 immutable public num;
    address constant public addr = 0x0000000000000000000000000000000000000000;

    constructor(uint256 _num) {
        num = _num;
    }
}",
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "state", 1),
        vec![4, 5]
    );
}

#[test]
fn mut_public() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("Immut"),
        String::from(
            "pragma solidity 0.8.0;

contract Immut {
    uint256 public num;
}",
        ),
    )]);

    assert!(!has_with_code(&findings, "state", 1));
}

// https://code4rena.com/reports/2022-10-zksync#gas-optimizations
#[test]
fn compound_state() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("Compound"),
        String::from(
            "pragma solidity 0.8.0;

contract Compound {
    uint private a;

    function moreExpensive() public {
        a += 1;
    }

    function moreExpensive2() public {
        a -= 1;
    }

    function lessExpensive1() public {
        a = a + 1;
    }

    function lessExpensive2() public {
        a = a - 1;
    }
}",
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "state", 3),
        vec![7, 11]
    );
}

// TODO
// this is indeed not a gas optimization, coumpound is cheaper for state only
#[test]
fn compound_non_state() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("Compound"),
        String::from(
            "pragma solidity 0.8.0;

contract Compound {
    function moreExpensive() public {
        uint a;
        a += 1;
    }

    function moreExpensive2() public {
        uint a;
        a -= 1;
    }

    function lessExpensive1() public {
        uint a;
        a = a + 1;
    }

    function lessExpensive2() public {
        uint a;
        a = a - 1;
    }
}",
        ),
    )]);

    assert!(!has_with_code(&findings, "state", 3),);
}

#[test]
fn bool_storage() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("Bool"),
        String::from(
            "pragma solidity 0.8.0;

contract Bool {
    bool public flip;
}",
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "state", 4),
        vec![4]
    );
}

#[test]
fn not_primitive_const() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("NotChange"),
        String::from(
            "pragma solidity 0.8.0;

contract NotChange {
    string public baseURI;
    mapping(uint256 => uint256) public map;

    constructor(string memory _baseURI) {
        baseURI = _baseURI;
    }
}",
        ),
    )]);

    assert!(!has_with_code(&findings, "state", 5));
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

    assert!(!has_with_code(&findings, "immutable", 5));
}

#[test]
fn immut() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("NotChange"),
        String::from(
            "pragma solidity 0.8.0;

contract Immut {
    uint256 immutable num;

    constructor(uint256 _num) {
        num = _num;
    }
}",
        ),
    )]);

    assert!(!has_with_code(&findings, "immutable", 5));
}

// dynamic types such as string and bytes cannot be declared as immutable
#[test]
fn not_primitive() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("Dynamic"),
        String::from(
            "pragma solidity 0.8.0;

contract Immut {
    string num;
}",
        ),
    )]);

    assert!(!has_with_code(&findings, "immutable", 5));
}

#[test]
fn const_var() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("Const"),
        String::from(
            "pragma solidity 0.8.0;

contract Const {
    uint256 constant num = 100;
}",
        ),
    )]);

    assert!(!has_with_code(&findings, "immutable", 5));
}
