// https://github.com/Picodes/4naly3er/blob/main/src/issues/L/avoidEncodePacked.ts

use crate::build_visitor;

build_visitor! {
    BTreeMap::from([
       (
           0,
           FindingKey {
               description: r#"`abi.encodePacked()` should not be used with dynamic types when passing the result to a hash function such as `keccak256()`. Use `abi.encode()` instead which will pad items to 32 bytes, which will [prevent hash collisions](https://docs.soliditylang.org/en/v0.8.13/abi-spec.html#non-standard-packed-mode) (e.g. `abi.encodePacked(0x123,0x456)` => `0x123456` => `abi.encodePacked(0x1,0x23456)`, but `abi.encode(0x123,0x456)` => `0x0...1230...456`). "Unless there is a compelling reason, `abi.encode` should be preferred". "#.to_string(),
               severity: Severity::Low
           }
       ),
       (
           1,
           FindingKey {
               description: "As there is only one argument to `abi.encodePacked()` it can often be cast to `bytes()` or `bytes32()` [instead](https://ethereum.stackexchange.com/questions/30912/how-to-compare-strings-in-solidity#answer-82739).\n".to_string(),
               severity: Severity::Low
           }
       ),
       (
           2,
           FindingKey {
               description: "As all arguments are strings and or bytes, `bytes.concat()` should be used instead".to_string(),
               severity: Severity::Low
           }
       )
    ]),
    fn visit_member_access(&mut self, member_access: &mut MemberAccess) {
        if member_access.member_name == "encodePacked" {
            let expression = &member_access.expression;
            if let Expression::Identifier(identifier) = expression {
            if identifier.name == "abi" {
                let mut dynamic = 0;
                member_access.argument_types.iter().for_each(|at| {
                    if let Some(type_string) = &at.type_string {
                        if type_string.starts_with("string") || type_string.starts_with("bytes") {
                            dynamic += 1;
                        }
                    }
                });

                self.push_finding(Some(member_access.src.clone()), 0);

                if dynamic == 0 {
                    // self.push_finding(Some(member_access.src.clone()), 0);
                } else if dynamic <= 1 {
                    self.push_finding(Some(member_access.src.clone()), 1);
                } else {
                    self.push_finding(Some(member_access.src.clone()), 2);
                }
            }
            }
        }

        member_access.visit(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
