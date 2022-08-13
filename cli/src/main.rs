use std::{env::current_dir, path::PathBuf};

use crate::{
    modules::{overflow, uint256},
    utils::formatter::format_findings,
    cmd::parse::Cmd,
};
use core::{solidity, walker::Walker};

use clap::Parser;

mod modules;
mod utils;
mod cmd;

fn main() {
    let args = Cmd::parse();

    let mut path = PathBuf::new();
    path.push(current_dir().expect("could not get current path"));
    path.push(args.path);

    let output = solidity::compile_artifacts(true, path);

    let module = uint256::get_module();

    let modules = vec![module];
    let mut walker = Walker::new(output, modules);

    let all_findings = walker.traverse().expect("failed to traverse ast");
    format_findings(all_findings);
}

#[cfg(test)]
mod test {
    use crate::modules::uint256;
    use crate::overflow;
    use core::walker::Walker;
    use ethers_solc::output::ProjectCompileOutput;
    use ethers_solc::project_util::TempProject;
    use semver::Version;

    fn compile_temp(name: impl AsRef<str>, content: impl AsRef<str>) -> ProjectCompileOutput {
        let tmp = TempProject::dapptools().unwrap();
        let f = tmp.add_contract(name, content).unwrap();
        tmp.project().compile_file(f.clone()).unwrap()
    }

    #[test]
    fn can_find_uint256() {
        let compiled = compile_temp(
            "examples/Foo",
            r#"
    pragma solidity ^0.8.10;
    contract Foo {
        uint256 unint;
    }
   "#,
        );

        assert!(!compiled.has_compiler_errors());
        assert!(compiled.find_first("Foo").is_some());

        let output = compiled.into_artifacts().collect();

        let modules = vec![uint256::get_module()];

        let mut walker = Walker::new(output, modules);
        let all_findings = walker.traverse().expect("couldn't");

        assert!(all_findings.get("uint256").unwrap().len() > 0)
    }

    #[test]
    fn dont_find_overflow() {
        let compiled = compile_temp(
            "examples/Foo",
            r#"
        pragma solidity ^0.8.10;
        contract Foo {
        mapping(address => uint256) bal;
            
            function deposit() external payable {
            bal[msg.sender] += msg.value;
            }
            
            function withdraw(uint256 amount) external {
            bal[msg.sender] -= amount;
            payable(msg.sender).transfer(amount);
            }
            
            fallback() external payable {}
        }
        "#,
        );

        assert!(!compiled.has_compiler_errors());
        assert!(compiled.find_first("Foo").is_some());

        let output = compiled.into_artifacts().collect();

        let modules = vec![overflow::get_module()];

        let mut walker = Walker::new(output, modules);
        let all_findings = walker.traverse().unwrap();

        assert_eq!(
            all_findings
                .get("overflow")
                .unwrap()
                .iter()
                .find_map(|mf| { (mf.finding.code == 0).then_some(true) }),
            None
        )
    }

    #[test]
    fn can_find_overflow_old_ver() {
        let compiled = compile_temp(
            "examples/Foo",
            r#"
        pragma solidity 0.7.0;
        contract Foo {
        mapping(address => uint256) bal;
            
            function deposit() external payable {
            bal[msg.sender] += msg.value;
            }
            
            function withdraw(uint256 amount) external {
            bal[msg.sender] -= amount;
            payable(msg.sender).transfer(amount);
            }
            
            fallback() external payable {}
        }
        "#,
        );

        assert!(!compiled.has_compiler_errors());
        assert!(compiled.find_first("Foo").is_some());

        let output = compiled.into_artifacts().collect();

        let modules = vec![overflow::get_module()];

        let mut walker = Walker::new(output, modules);
        let all_findings = walker.traverse().unwrap();

        assert_eq!(
            all_findings
                .get("overflow")
                .unwrap()
                .iter()
                .find_map(|mf| { (mf.finding.code == 0).then_some(true) })
                .unwrap(),
            true
        )
    }
}
