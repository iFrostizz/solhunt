use crate::build_visitor;

build_visitor!(
    BTreeMap::from([
        (
            0,
            FindingKey {
                description: "Initializers could be front-run, allowing an attacker to either set their own values, take ownership of the contract, and in the best case forcing a re-deployment".to_string(),
                summary: "Front-run of initializers".to_string(),
                severity: Severity::Low
            }
        ),
        (
            1,
            FindingKey {
                description: "Initialized function does not have an initializable modifier".to_string(),
                summary: "Missing modifier".to_string(),
                severity: Severity::Medium
            }
        ),
    ]),
    fn visit_contract_definition(&mut self, contract_definition: &mut ContractDefinition) {
        
        match contract_definition.kind {
            // don't even visit it if it's an interface
            ContractKind::Interface => Ok(()),
            _ => contract_definition.visit(self)
        }
    },

    fn visit_function_definition(&mut self, function_definition: &mut FunctionDefinition) {
        if function_definition.name == "initialize" {
            let has_initializer = function_definition.modifiers.iter().any(|modifier| {
                if let IdentifierOrIdentifierPath::IdentifierPath(id) = &modifier.modifier_name {
                    id.name == "initializer"
                } else {false}
            });

            if has_initializer {
                self.push_finding(0, Some(function_definition.src.clone()));
            } else {
                self.push_finding(1, Some(function_definition.src.clone()));
            }
        }

        function_definition.visit(self)
    }
);

#[test]
fn initialize_function() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("Initialize"),
        String::from(
            "pragma solidity 0.8.0;

contract Initialize {
    bool initialized;

    modifier initializer() {
        require(!initialized);
        initialized = true;
        _;
    }

    function initialize() public initializer {
        // Initialize some proxy
    }
}",
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "proxy", 0),
        vec![12]
    );
}

#[test]
fn missing_initializer() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("MissingInitialize"),
        String::from(
            "pragma solidity 0.8.0;

contract MissingInitialize {
    function initialize() public {
        // Initialize some proxy
    }
}",
        ),
    )]);

    assert_eq!(lines_for_findings_with_code_module(&findings, "proxy", 1), vec![4]);
}

#[test]
fn not_interfaces() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("MissingInitialize"),
        String::from(
            "pragma solidity 0.8.0;

interface MissingInitialize {
    function initialize() external;
}",
        ),
    )]);

    assert!(!has_with_code(&findings, "proxy", 1));
}
