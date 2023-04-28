use super::{build_test, Felt};
use vm_core::utils::{group_slice_elements, IntoBytes};

#[test]
fn blake3_hash_64_bytes() {
    let source = "
    use.std::crypto::hashes::blake3

    begin
        exec.blake3::hash_2to1
    end
    ";

    let input0 = rand_utils::rand_array::<Felt, 4>().into_bytes();
    let input1 = rand_utils::rand_array::<Felt, 4>().into_bytes();

    let mut ibytes = [0u8; 64];
    ibytes[..32].copy_from_slice(&input0);
    ibytes[32..].copy_from_slice(&input1);

    let ifelts = group_slice_elements::<u8, 4>(&ibytes)
        .iter()
        .map(|&bytes| u32::from_le_bytes(bytes) as u64)
        .rev()
        .collect::<Vec<u64>>();

    let hasher = blake3::hash(&ibytes);
    let obytes = hasher.as_bytes();
    let ofelts = group_slice_elements::<u8, 4>(obytes)
        .iter()
        .map(|&bytes| u32::from_le_bytes(bytes) as u64)
        .collect::<Vec<u64>>();

    let test = build_test!(source, &ifelts);
    test.expect_stack(&ofelts);
}

#[test]
fn blake3_hash_32_bytes() {
    let source = "
    use.std::crypto::hashes::blake3

    begin
        exec.blake3::hash_1to1
    end
    ";

    let ibytes = rand_utils::rand_array::<Felt, 4>().into_bytes();
    let ifelts = group_slice_elements::<u8, 4>(&ibytes)
        .iter()
        .map(|&bytes| u32::from_le_bytes(bytes) as u64)
        .rev()
        .collect::<Vec<u64>>();

    let hasher = blake3::hash(&ibytes);
    let obytes = hasher.as_bytes();
    let ofelts = group_slice_elements::<u8, 4>(obytes)
        .iter()
        .map(|&bytes| u32::from_le_bytes(bytes) as u64)
        .collect::<Vec<u64>>();

    let test = build_test!(source, &ifelts);
    test.expect_stack(&ofelts);
}
