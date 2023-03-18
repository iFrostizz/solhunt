// https://github.com/code-423n4/2022-12-tigris-findings/blob/main/data/Deekshith99-G.md 3rd and 4th

use crate::{
    build_visitor,
    utils::{tightly_pack, type_as_bytes},
};

#[cfg(test)]
use crate::{solidity::compile_single_contract_to_artifacts, walker::Walker};
#[cfg(test)]
use std::{cell::RefCell, ops::Deref, rc::Rc};

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
        if let Some(packed) = tightly_pack(struct_bytes.clone()) {
            let packed_struct = propose_better_packing(struct_definition, struct_bytes.into_iter().flatten().collect(), packed.into_iter().flatten().collect());

            let repr = struct_to_sol_representation(&packed_struct);

            self.push_finding_comment(0, Some(struct_definition.src.clone()), repr);
        };

        struct_definition.visit(self)
    }
}

pub fn extract_struct_bytes(struct_definition: StructDefinition) -> Vec<Vec<usize>> {
    let mut struct_bytes = Vec::new();
    let mut local_bytes = Vec::new();

    struct_definition
        .members
        .iter()
        .enumerate()
        .for_each(|(i, m)| {
            dbg!(&m);
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

    struct_bytes
}

/// rearrange the struct members to match the tight representation of it in bytes
pub fn propose_better_packing(
    struc: &StructDefinition,
    mut loose: Vec<usize>,
    tight: Vec<usize>,
) -> StructDefinition {
    let mut packed_struc = struc.clone();
    let mut members = struc.members.clone();

    while loose != tight {
        for i in 0..loose.len() {
            if loose[i] != tight[i] {
                // get the next index whose value is the current one to swap it
                let next_id = tight
                    .iter()
                    .enumerate()
                    .find(|(_, &x)| x == loose[i])
                    .map(|(j, _)| j)
                    .unwrap();

                // update the mirrored vec as well as the struct
                loose.swap(i, next_id);
                members.swap(i, next_id);

                //                 dbg!(&loose);
                //                 dbg!(&tight);
            }
        }
    }

    packed_struc.members = members.to_vec();

    packed_struc
}

pub fn struct_to_sol_representation(struc: &StructDefinition) -> String {
    let mut rep = String::new();

    rep.push_str(&format!("```solidity\nstruct {} {{\n", struc.name));

    struc.members.iter().for_each(|mem| {
        rep.push_str(&format!(
            "   {} {};\n",
            mem.type_descriptions.type_string.clone().unwrap(),
            mem.name,
        ))
    });

    rep.push_str("}\n```");

    rep
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
        lines_for_findings_with_code_module(&findings, "tight_pack", 0),
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

#[derive(Default)]
pub struct StructModule {
    struct_name: Option<String>,
    expected_struct_bytes: Vec<Vec<usize>>,
    tight_struct_bytes: Option<Vec<Vec<usize>>>,
    shared_data: ModuleState,
}

impl StructModule {
    #[cfg(test)]
    fn new(expected_struct_bytes: Vec<Vec<usize>>) -> Self {
        Self {
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
        if let Some(name) = &self.struct_name {
            if name != &struct_definition.name {
                return struct_definition.visit(self);
            }
        }

        let struct_bytes = extract_struct_bytes(struct_definition.clone());

        assert_eq!(
            struct_definition.members.len(),
            struct_bytes.iter().flatten().collect::<Vec<&usize>>().len()
        );
        assert_eq!(struct_bytes, self.expected_struct_bytes);

        self.tight_struct_bytes = tightly_pack(struct_bytes);

        struct_definition.visit(self)
    }
}

#[test]
fn extract_types_from_struct() {
    let (_project, artifacts) = compile_single_contract_to_artifacts(String::from(
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
        vec![Rc::from(RefCell::from(module))],
    );

    walker.traverse().unwrap();

    let (_project, artifacts) = compile_single_contract_to_artifacts(String::from(
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
        vec![Rc::from(RefCell::from(module))],
    );

    walker.traverse().unwrap();

    let (_project, artifacts) = compile_single_contract_to_artifacts(String::from(
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
        vec![Rc::from(RefCell::from(module))],
    );

    walker.traverse().unwrap();
}

#[test]
fn can_pack() {
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("Looz"),
        String::from(
            "pragma solidity 0.8.0;

interface IExecutor {
    struct StoredBlockInfo {
        uint64 blockNumber;
        bytes32 blockHash;
        uint64 indexRepeatedStorageChanges;
        uint256 numberOfLayer1Txs;
        bytes32 priorityOperationsHash;
        bytes32 l2LogsTreeRoot;
        uint256 timestamp;
        bytes32 commitment;
    }
}",
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code_module(&findings, "tight_pack", 0),
        vec![4]
    );
}

// https://code4rena.com/reports/2022-04-jpegd/#g-16-nftvaultsol-struct-positionpreview-can-be-tightly-packed-to-save-1-storage-slot
#[test]
fn jpeg_sale() {
    let (_project, artifacts) = compile_single_contract_to_artifacts(String::from(
        "pragma solidity 0.8.0;

enum BorrowType {
    NOT_CONFIRMED,
    NON_INSURANCE,
    USE_INSURANCE
}

struct Rate {
    uint128 numerator;
    uint128 denominator;
}

struct VaultSettings {
    Rate debtInterestApr;
    Rate creditLimitRate;
    Rate liquidationLimitRate;
    Rate valueIncreaseLockRate;
    Rate organizationFeeRate;
    Rate insurancePurchaseRate;
    Rate insuranceLiquidationPenaltyRate;
    uint256 insuraceRepurchaseTimeLimit;
    uint256 borrowAmountCap;
}

struct PositionPreview { // @audit gas: can be tightly packed by moving borrowType and liquidatable at the end
    address owner;
    uint256 nftIndex;
    bytes32 nftType;
    uint256 nftValueUSD;
    VaultSettings vaultSettings;
    uint256 creditLimit;
    uint256 debtPrincipal;
    uint256 debtInterest; // @audit gas: 32 bytes
    BorrowType borrowType; // @audit gas: 1 byte (this enum is equivalent to uint8 as it has less than 256 options)
    bool liquidatable; // @audit gas: 1 byte
    uint256 liquidatedAt; // @audit gas: 32 bytes
    address liquidator; // @audit gas: 20 bytes
}
"));

    let module: Rc<RefCell<dyn Visitor<ModuleState>>> = Rc::from(RefCell::from(StructModule {
        struct_name: Some(String::from("PositionPreview")),
        expected_struct_bytes: vec![
            vec![20],
            vec![32],
            vec![32],
            vec![32],
            vec![16, 16],
            vec![16, 16],
            vec![16, 16],
            vec![16, 16],
            vec![16, 16],
            vec![16, 16],
            vec![16, 16],
            vec![16, 16],
            vec![16, 16],
            vec![32],
            vec![32],
            vec![32],
            vec![32],
            vec![32],
            vec![1, 1],
            vec![32],
            vec![20],
        ],
        ..Default::default()
    }));

    let mut walker = Walker::new(artifacts, BTreeMap::new(), vec![module.clone()]);

    walker.traverse().unwrap();

    let mut_mod = module.borrow_mut();
    let der_mod = mut_mod.deref();
    let struct_mod = der_mod.as_any().downcast_ref::<StructModule>().unwrap();

    assert_eq!(
        struct_mod.tight_struct_bytes.clone().unwrap(),
        vec![vec![6, 6, 20], vec![10, 10, 10], vec![12, 20], vec![12],]
    );
}
