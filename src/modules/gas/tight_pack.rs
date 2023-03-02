// https://github.com/code-423n4/2022-12-tigris-findings/blob/main/data/Deekshith99-G.md 3rd and 4th

use crate::{
    build_visitor,
    utils::{tightly_pack, type_as_bytes},
};

#[cfg(test)]
use crate::{solidity::compile_single_contract_to_artifacts, walker::Walker};

build_visitor! {
    BTreeMap::from([
        (
            0,
            FindingKey {
                summary: "Variables can be tightly packed".to_string(),
                description: "Variables using less than one slot can be tighly packed in order to save gas when loading them".to_string(),
                severity: Severity::Gas
            }
        )
    ]),

    fn visit_struct_definition(&mut self, struct_definition: &mut StructDefinition) {
        let struct_bytes = extract_struct_bytes(struct_definition.clone());

        // TODO: propose a better packing in a "comment" section
        if let Some(_packed) = tightly_pack(struct_bytes) {
            self.push_finding(0, Some(struct_definition.src.clone()));
        };

        struct_definition.visit(self)
    }
}

// TODO: be able to extract a specific node from the ast to unit test this function
pub fn extract_struct_bytes(struct_definition: StructDefinition) -> Vec<Vec<usize>> {
    let mut struct_bytes = Vec::new();
    let mut local_bytes = Vec::new();

    struct_definition
        .members
        .iter()
        .enumerate()
        .for_each(|(i, m)| {
            let bytes = type_as_bytes(&m.type_descriptions.type_string.clone().unwrap());

            // copy the solidity behaviour, only pack variables next to each other
            if local_bytes.iter().sum::<usize>() + bytes <= 32 {
                // if slot will still not be filled, keep pushing in the cache vec
                local_bytes.push(bytes);
            } else {
                struct_bytes.push(local_bytes.clone());
                local_bytes.clear();
                // start the new cache vec
                local_bytes.push(bytes);
            }

            // empty the cache vec if it's the last run
            if i == struct_definition.members.len() - 1 && !local_bytes.is_empty() {
                struct_bytes.push(local_bytes.clone());
            }
        });

    // Remove any empty element
    // struct_bytes.into_iter().filter(|b| !b.is_empty()).collect()
    struct_bytes
}

#[test]
fn loose_struct() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("LooseStruct"),
        String::from(
            "pragma solidity 0.8.0;

contract LooseStruct {
    struct ImLoose {
        uint full_1;
        address part_stuff;
        uint256 another_full;
        bool waste_of_space;
    }
}",
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code(&findings, "tight_pack", 0),
        vec![4]
    );
}

#[test]
fn tight_struct() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("TightStruct"),
        String::from(
            "pragma solidity 0.8.0;

contract TightStruct {
    struct MemoryUserOp {
        address sender;
        uint256 nonce;
        uint256 callGasLimit;
        uint256 verificationGasLimit;
        uint256 preVerificationGas;
        address paymaster;
        uint256 maxFeePerGas;
        uint256 maxPriorityFeePerGas;
    }
}",
        ),
    )]);

    assert!(!has_with_code(&findings, "tight_pack", 0));
}

// TODO: write a dummy visitor for testing purposes which does not implements it with Vec<Finding> as Data
#[derive(Default)]
pub struct StructModule {
    findings: Vec<Finding>,
    expected_struct_bytes: Vec<Vec<usize>>,
    shared_data: ModuleState,
}

impl StructModule {
    #[cfg(test)]
    fn new(expected_struct_bytes: Vec<Vec<usize>>) -> Self {
        Self {
            findings: vec![],
            expected_struct_bytes,
            ..Default::default()
        }
    }
}

impl Visitor<ModuleState> for StructModule {
    fn shared_data(&mut self) -> &ModuleState {
        &self.shared_data
    }

    fn visit_struct_definition(
        &mut self,
        struct_definition: &mut StructDefinition,
    ) -> eyre::Result<(), VisitError> {
        let struct_bytes = extract_struct_bytes(struct_definition.clone());

        assert_eq!(struct_bytes, self.expected_struct_bytes);

        struct_definition.visit(self)
    }
}

#[test]
fn extract_types_from_struct() {
    let (project, artifacts) = compile_single_contract_to_artifacts(String::from(
        "pragma solidity 0.8.0;

struct MyStruct {
    uint256 val_1;
    address user;
    uint8 lil;
}",
    ));

    let module = StructModule::new(vec![vec![32], vec![20, 1]]);

    let mut walker = Walker::new(
        artifacts,
        BTreeMap::new(),
        vec![Box::from(module)],
        project.root().into(),
    );

    walker.traverse().unwrap();

    let (project, artifacts) = compile_single_contract_to_artifacts(String::from(
        "pragma solidity 0.8.0;

struct MyStruct {
    address sender;
    uint256 nonce;
    uint256 callGasLimit;
    uint256 verificationGasLimit;
    uint256 preVerificationGas;
    address paymaster;
    uint256 maxFeePerGas;
    uint256 maxPriorityFeePerGas;
}",
    ));

    let module = StructModule::new(vec![
        vec![20],
        vec![32],
        vec![32],
        vec![32],
        vec![32],
        vec![20],
        vec![32],
        vec![32],
    ]);

    let mut walker = Walker::new(
        artifacts,
        BTreeMap::new(),
        vec![Box::from(module)],
        project.root().into(),
    );

    walker.traverse().unwrap();

    let (project, artifacts) = compile_single_contract_to_artifacts(String::from(
        "pragma solidity 0.8.0;

struct MyStruct {

    address sender;
    uint256 nonce;
    bytes initCode;
    bytes callData;
    uint256 callGasLimit;
    uint256 verificationGasLimit;
    uint256 preVerificationGas;
    uint256 maxFeePerGas;
    uint256 maxPriorityFeePerGas;
    bytes paymasterAndData;
    bytes signature;
}",
    ));

    let module = StructModule::new(vec![
        vec![20],
        vec![32],
        vec![32],
        vec![32],
        vec![32],
        vec![32],
        vec![32],
        vec![32],
        vec![32],
        vec![32],
        vec![32],
    ]);

    let mut walker = Walker::new(
        artifacts,
        BTreeMap::new(),
        vec![Box::from(module)],
        project.root().into(),
    );

    walker.traverse().unwrap();
}
