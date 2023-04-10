# How to build a detection module

## Intro

This repo currently relies on a fork of ethers-rs with new types for more granularity and to avoid too much repetition as well as a "visitor" pattern. The `Visitor` is a trait that can be implemented by the detection modules which can hold some arbitrary state (including the findings). Everytime it visits a node, it will call the underlying function back to the implementer of the trait to notify that this node is being visited.

The `build_visitor!` macro was made to avoid having to copy and paste the same implementation over and over so that you can focus on the findings.

See this dummy detection module, which can find all declaration of an uint256:

```rust
// A silly module that finds all uint256

use crate::build_visitor;

build_visitor!(
    BTreeMap::from([(
        0,
        FindingKey {
            description: "We just found a uint256 yay!".to_string(),
            severity: Severity::Informal
        }
    )]),
    fn visit_variable_declaration(&mut self, var: &mut VariableDeclaration) {
        if let Some(type_id) = &var.type_descriptions.type_identifier {
            if type_id == "t_uint256" {
                self.push_finding(Some(var.src.clone()), 0);
            }
        }

        var.visit(self)
    }
);
```

## What's going on ?

First, we are importing the `build_visitor!` macro.
Inside of this macro, we can define the findings, and how to actually find them.
We are creating a map that links the finding code to a more detailed description as well as the severity. Finding a `uint256` may seem critical, but it seems more apropriate to define it as `Informal`.

In the implementation, we are defining `visit_variable_declaration()` which will be called by the Visitor. If the `type identifier` is a "t_uint256", then we can raise the finding.

We are then calling `var.visit(self)` to notify that we should keep visiting the nodes nested in the `variable_declaration`.

You can do the choice to omit it, and just return `Ok(())` if you believe that there won't be any other function relying on some of these nested nodes. It's quite easy to forget it though and can be somewhat hard to understand why some functions (in nested nodes) aren't being visited and maybe not really worth the optimization.

## Avoiding false positives

Now that the detection logic is written, we can test the module.

```rust
#[cfg(test)]
mod test {
    use crate::{
        solidity::ProjectFile,
        test::{compile_and_get_findings, lines_for_findings_with_code},
    };

    #[test]
    fn can_find_dummy_uint256() {
        let findings = compile_contract_and_get_findings(vec![ProjectFile::Contract(
            String::from(
                "pragma solidity 0.8.0;

            contract DummyUint256 {
                uint256 unint;
            }",
            ),
        )]);

        assert_eq!(
            lines_for_findings_with_code(&findings, "uint256", 0),
            vec![4]
        );
    }
}
```

We are writing a contract, which will be saved on a temporarly location on your filesystem when running the test. This is to make sure that it's going to work similarly when running the binary.

After catching the findings, we are asserting that the location of this `uint256` declaration which has the finding code "0" is at line 4.

Making sure that tests passes for each situation will reduce the amount of false positives as well as false negatives.

