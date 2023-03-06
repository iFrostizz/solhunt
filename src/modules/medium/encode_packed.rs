// https://github.com/Picodes/4naly3er/blob/main/src/issues/L/avoidEncodePacked.ts

use crate::build_visitor;

build_visitor! {
    BTreeMap::from([
       (
           0,
           FindingKey {
               description: r#"`abi.encodePacked()` should not be used with dynamic types when passing the result to a hash function such as `keccak256()`. Use `abi.encode()` instead which will pad items to 32 bytes, which will [prevent hash collisions](https://docs.soliditylang.org/en/v0.8.13/abi-spec.html#non-standard-packed-mode) (e.g. `abi.encodePacked(0x123,0x456)` => `0x123456` => `abi.encodePacked(0x1,0x23456)`, but `abi.encode(0x123,0x456)` => `0x0...1230...456`). "Unless there is a compelling reason, `abi.encode` should be preferred". "#.to_string(),
               summary: "Usage of abi.encodePacked() with dynamic types".to_string(),
               severity: Severity::Low
           }
       ),
       (
           1,
           FindingKey {
               description: "As there is only one argument to `abi.encodePacked()` it can often be cast to `bytes()` or `bytes32()` [instead](https://ethereum.stackexchange.com/questions/30912/how-to-compare-strings-in-solidity#answer-82739).\n".to_string(),
               summary: "One argument with abi.encodePacked()".to_string(),
               severity: Severity::Low
           }
       ),
       (
           2,
           FindingKey {
               description: "As all arguments are strings and or bytes, `bytes.concat()` should be used instead".to_string(),
               summary: "Only strings and bytes".to_string(),
               severity: Severity::Low
           }
       ),
       (
           3,
           FindingKey {
                summary: "`abi.encode()` is less efficient than `abi.encodepacked()`".to_string(),
                description: "see: https://github.com/ConnorBlockchain/Solidity-Encode-Gas-Comparison".to_string(),
                severity: Severity::Gas
           }
       )
    ]),

    fn visit_member_access(&mut self, member_access: &mut MemberAccess) {
        // dbg!(&member_access);
        if let Expression::Identifier(identifier) = &member_access.expression {
            if identifier.name == "abi" && identifier.type_descriptions.type_string == Some("abi".to_string()) {
                if member_access.member_name == "encodePacked" {
                    let mut dynamic = 0;
                    member_access.argument_types.iter().for_each(|at| {
                        if let Some(type_string) = &at.type_string {
                            if type_string.starts_with("string") /*|| type_string.starts_with("bytes")*/ {
                                dynamic += 1;
                            }
                        }
                    });

                    // println!("a");
                    self.push_finding(0, Some(member_access.src.clone()));

                    if dynamic == 0 {
                        // self.push_finding(Some(member_access.src.clone()), 0);
                    } else if dynamic <= 1 {
                        self.push_finding(1, Some(member_access.src.clone()));
                    } else {
                        self.push_finding(2, Some(member_access.src.clone()));
                    }
                } else if member_access.member_name == "encode" {
                    self.push_finding(3, Some(member_access.src.clone()));
                }
            }
        }

        member_access.visit(self)
    }
}

#[test]
fn encode_packed_collision() {
    // inspired from: https://github.com/sherlock-audit/2022-10-nftport-judging/issues/118
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("EncodePacked"),
        String::from(
            "pragma solidity ^0.8.0;
contract EncodePacked {
    modifier signedOnly(bytes memory message, bytes memory signature) {
        // do some amazing checks
        _;
    }

    function deploy(
        string calldata templateName,
        bytes calldata initdata,
        bytes calldata signature
    )
        external
        payable
        signedOnly(
            abi.encodePacked(msg.sender, templateName, initdata),
            signature
        )
    {
        // _deploy(templateName, latestVersion[templateName], initdata);
    }
}",
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code(&findings, "encode_packed", 0),
        vec![16]
    );
}

#[test]
fn encode_less_efficient() {
    // inspired from: https://github.com/code-423n4/2023-01-biconomy-findings/blob/main/data/Rolezn-G.md#gas1-abiencode-is-less-efficient-than-abiencodepacked
    let findings = compile_and_get_findings(vec![ProjectFile::Contract(
        String::from("Encode"),
        String::from(
            "pragma solidity ^0.8.0;

contract Encode {
    function encode(bytes32 clear) public returns (bytes memory) {
        return abi.encode(clear);
    }
}",
        ),
    )]);

    assert_eq!(
        lines_for_findings_with_code(&findings, "encode_packed", 3),
        vec![5]
    );
}
