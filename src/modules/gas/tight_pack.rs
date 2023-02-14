// https://github.com/code-423n4/2022-12-tigris-findings/blob/main/data/Deekshith99-G.md 3rd and 4th

use crate::{
    build_visitor,
    utils::{tightly_pack, type_as_bytes},
};

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

        // TODO: propose a better packing
        if let Some(_packed) = tightly_pack(struct_bytes) {
            self.push_finding(0, Some(struct_definition.src.clone()));
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
            let bytes = type_as_bytes(&m.type_descriptions.type_string.clone().unwrap());

            if local_bytes.iter().sum::<usize>() + bytes <= 32 {
                local_bytes.push(bytes);
            } else {
                struct_bytes.push(local_bytes.clone());
                local_bytes.clear();
            }

            if i == struct_definition.members.len() - 1 {
                struct_bytes.push(local_bytes.clone());
            }
        });

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
