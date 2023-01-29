use crate::build_visitor;

build_visitor!(
    BTreeMap::from([
        (
            0,
            FindingKey {
                description: "usage of deprecated chainlink oracle feed function".to_string(),
                severity: Severity::Medium,
            }
        ),
        (
            1,
            FindingKey {
                description: "stale price from chainlink oracle".to_string(),
                severity: Severity::Medium,
            }
        )
    ]),
    fn visit_member_access(&mut self, member_access: &mut MemberAccess) {
        if let Some(id) = &member_access.type_descriptions.type_identifier {
            if id.ends_with("returns$_t_int256_$") && member_access.member_name == "latestAnswer" {
                self.push_finding(Some(member_access.src.clone()), 0)
            } else if id
                .ends_with("returns$_t_uint80_$_t_int256_$_t_uint256_$_t_uint256_$_t_uint80_$")
                && member_access.member_name == "latestRoundData"
            {
                self.push_finding(Some(member_access.src.clone()), 1)
            }
        }

        member_access.visit(self)
    }
);

#[cfg(test)]
mod tests {
    use crate::{
        solidity::ProjectFile,
        test::{compile_and_get_findings, lines_for_findings_with_code},
    };

    #[test]
    fn deprecated_chainlink_feed() {
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("DeprecatedChainlink"),
            String::from(
                "pragma solidity 0.8.0;

interface AggregatorInterface {
  function latestAnswer() external view returns (int256);
}

contract DeprecatedChainlink {
    function getPrice(AggregatorInterface oracle) public view returns (int256) {
        int256 price = oracle.latestAnswer();
        return price;
    }
}",
            ),
        )]);

        assert_eq!(
            lines_for_findings_with_code(&findings, "chainlink", 0),
            vec![9]
        );
    }

    // https://github.com/code-423n4/2022-04-jpegd-findings/issues/54
    // https://github.com/code-423n4/2021-12-yetifinance-findings/issues/91
    #[test]
    fn stale_price_no_get() {
        let findings = compile_and_get_findings(vec![ProjectFile::Contract(
            String::from("StalePrice"),
            String::from(
                "pragma solidity 0.8.0;

interface AggregatorInterface {
  function latestRoundData() external view returns (uint80 roundId, int256 answer, uint256 startedAt, uint256 updatedAt, uint80 answeredInRound);
}

contract StalePrice {
    function getPrice(AggregatorInterface oracle) public view returns (int256) {
        (,int256 price, , ,) = oracle.latestRoundData();
        return price;
    }
}",
            ),
        )]);

        assert_eq!(
            lines_for_findings_with_code(&findings, "chainlink", 1),
            vec![9]
        );
    }
}
