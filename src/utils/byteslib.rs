use itertools::Itertools;

pub fn type_as_bytes(id: &str) -> usize {
    // from ethers_core / lexer

    match id {
        "address" => 20,
        "address payable" => 20,
        "bool" => 1,
        "bytes1" => 1,
        "bytes2" => 2,
        "bytes3" => 3,
        "bytes4" => 4,
        "bytes5" => 5,
        "bytes6" => 6,
        "bytes7" => 7,
        "bytes8" => 8,
        "bytes9" => 9,
        "bytes10" => 10,
        "bytes11" => 11,
        "bytes12" => 12,
        "bytes13" => 13,
        "bytes14" => 14,
        "bytes15" => 15,
        "bytes16" => 16,
        "bytes17" => 17,
        "bytes18" => 18,
        "bytes19" => 19,
        "bytes20" => 20,
        "bytes21" => 21,
        "bytes22" => 22,
        "bytes23" => 23,
        "bytes24" => 24,
        "bytes25" => 25,
        "bytes26" => 26,
        "bytes27" => 27,
        "bytes28" => 28,
        "bytes29" => 29,
        "bytes30" => 30,
        "bytes31" => 31,
        "bytes32" => 32,
        "int8" => 1,
        "int16" => 2,
        "int24" => 3,
        "int32" => 4,
        "int40" => 5,
        "int48" => 6,
        "int56" => 7,
        "int64" => 8,
        "int72" => 9,
        "int80" => 10,
        "int88" => 11,
        "int96" => 12,
        "int104" => 13,
        "int112" => 14,
        "int120" => 15,
        "int128" => 16,
        "int136" => 17,
        "int144" => 18,
        "int152" => 19,
        "int160" => 20,
        "int168" => 21,
        "int176" => 22,
        "int184" => 23,
        "int192" => 24,
        "int200" => 25,
        "int208" => 26,
        "int216" => 27,
        "int224" => 28,
        "int232" => 29,
        "int240" => 30,
        "int248" => 31,
        "int256" => 32,
        "int" => 32,
        "uint8" => 1,
        "uint16" => 2,
        "uint24" => 3,
        "uint32" => 4,
        "uint40" => 5,
        "uint48" => 6,
        "uint56" => 7,
        "uint64" => 8,
        "uint72" => 9,
        "uint80" => 10,
        "uint88" => 11,
        "uint96" => 12,
        "uint104" => 13,
        "uint112" => 14,
        "uint120" => 15,
        "uint128" => 16,
        "uint136" => 17,
        "uint144" => 18,
        "uint152" => 19,
        "uint160" => 20,
        "uint168" => 21,
        "uint176" => 22,
        "uint184" => 23,
        "uint192" => 24,
        "uint200" => 25,
        "uint208" => 26,
        "uint216" => 27,
        "uint224" => 28,
        "uint232" => 29,
        "uint240" => 30,
        "uint248" => 31,
        "uint256" => 32,
        "uint" => 32,
        _ => 32,
    }
}

/// Finds all permutations and returns the most tightly packed one
pub fn tightly_pack(loose_bytes: Vec<Vec<usize>>) -> Option<Vec<Vec<usize>>> {
    loose_bytes
        .iter()
        .for_each(|bs| assert!(bs.iter().sum::<usize>() <= 32));

    // ignore 32 slots
    let bytes: Vec<Vec<_>> = loose_bytes
        .clone()
        .into_iter()
        .map(|bs| bs.into_iter().filter(|b| *b < 32).collect::<Vec<usize>>())
        .filter(|bs| !bs.is_empty())
        .collect();

    let loose_len = loose_bytes.len();
    let bytes_len = bytes.len();

    let stipend_len = loose_len - bytes_len;
    let mut stipend = vec![vec![32]; stipend_len];

    let mut packed: Option<Vec<Vec<usize>>> = None;

    let bytes: Vec<_> = bytes.into_iter().flatten().collect();
    let len = bytes.len();

    let mut smollest = loose_len;

    for perm in bytes.into_iter().permutations(len).unique() {
        let mut local_packed = pack(perm);
        let per_len = local_packed.len() + stipend_len;

        // get the most packed one
        if per_len < smollest {
            local_packed.append(&mut stipend);
            packed = Some(local_packed);
            smollest = per_len;
        }
    }

    packed
}

/// Looks lineary in the Vec and packs bytes
fn pack(bytes: Vec<usize>) -> Vec<Vec<usize>> {
    let mut packed = Vec::new();

    let mut current_bytes: Vec<usize> = Vec::new();

    for (i, byte) in bytes.iter().enumerate() {
        if current_bytes.clone().into_iter().sum::<usize>() + byte <= 32 {
            current_bytes.push(*byte);
        } else {
            packed.push(current_bytes.clone());
            current_bytes = vec![*byte];
        }

        // don't forget the last elements at the end
        if i == bytes.len() - 1 {
            packed.push(current_bytes.clone());
        }
    }

    packed
}

#[test]
fn packs() {
    assert_eq!(pack(vec![32, 8, 20]), vec![vec![32], vec![8, 20]]);
}

#[test]
fn packs_tightly() {
    assert_eq!(
        tightly_pack(vec![vec![32], vec![20], vec![32], vec![8]]),
        Some(vec![vec![20, 8], vec![32], vec![32]])
    );
}
