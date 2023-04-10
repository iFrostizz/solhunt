// a bench test that checks for all single modules speed on cached artifacts

use std::time::Duration;
use std::{collections::BTreeMap, rc::Rc};

use criterion::{criterion_group, criterion_main, Criterion};
use ethers_solc::artifacts::Optimizer;
use solhunt::{loader::get_all_visitors, solidity::compile_path_to_artifacts, walker::Walker};

pub fn biconomy_bench(c: &mut Criterion) {
    let visitors = get_all_visitors();
    let artifacts = compile_path_to_artifacts(
        "test-data/biconomy/",
        Some(Optimizer {
            enabled: Some(true),
            runs: Some(200),
            details: None,
        }),
    )
    .unwrap();

    visitors.iter().for_each(|v| {
        let name = {
            let cvi = v.clone();
            let visitor = Rc::clone(&cvi);
            let mut cvis = visitor.borrow_mut();
            let data = cvis.shared_data();

            &data.name.clone()
        };

        c.bench_function(&format!("biconomy_{}", name), |b| {
            let mut walker =
                Walker::new(artifacts.clone(), BTreeMap::default(), vec![Rc::clone(v)]);

            b.iter(|| walker.traverse().unwrap());
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(13));
    targets = biconomy_bench
}
criterion_main!(benches);
