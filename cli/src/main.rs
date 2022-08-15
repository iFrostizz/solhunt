use crate::{cmd::parse::parse, utils::formatter::format_findings};
use core::{solidity, walker::Walker};

mod cmd;
mod modules;
mod utils;

fn main() {
    let (path, loader, verbosity) = parse();

    let output = solidity::compile_artifacts(true, path);

    let mut walker = Walker::new(output, loader);

    let all_findings = walker.traverse().expect("failed to traverse ast");
    format_findings(all_findings, verbosity);
}

#[cfg(test)]
mod test {
    use crate::modules::loader::get_all_modules;
    use core::{
        loader::Loader,
        walker::{AllFindings, MetaFinding, Walker},
    };
    use ethers_solc::{output::ProjectCompileOutput, project_util::TempProject};

    fn compile_temp(name: impl AsRef<str>, content: impl AsRef<str>) -> ProjectCompileOutput {
        let tmp = TempProject::dapptools().unwrap();
        let f = tmp.add_contract(name, content).unwrap();
        tmp.project().compile_file(f.clone()).unwrap()
    }

    fn has_with_code(name: &str, code: u32, all_findings: &AllFindings) -> bool {
        all_findings
            .get(name)
            .unwrap()
            .iter()
            .any(|mf| mf.finding.code == code)
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

        let modules = get_all_modules();
        let loader = Loader::new(modules);

        let mut walker = Walker::new(output, loader);
        let all_findings = walker.traverse().expect("couldn't");

        assert!(!all_findings.get("uint256").unwrap().is_empty())
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

        let modules = get_all_modules();
        let loader = Loader::new(modules);

        let mut walker = Walker::new(output, loader);
        let all_findings = walker.traverse().unwrap();

        assert!(!has_with_code("overflow", 0, &all_findings));
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

        let modules = get_all_modules();
        let loader = Loader::new(modules);

        let mut walker = Walker::new(output, loader);
        let all_findings = walker.traverse().unwrap();

        assert!(has_with_code("overflow", 0, &all_findings));
        assert!(has_with_code("overflow", 1, &all_findings));
    }
}
