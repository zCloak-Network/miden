use proptest::prelude::*;
use vm_core::{stack::STACK_TOP_SIZE, WORD_SIZE};

use crate::build_op_test;
use crate::helpers::TestError;

// STACK OPERATIONS TESTS
// ================================================================================================

#[test]
fn drop() {
    let asm_op = "drop";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_stack(&[2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 0]);
}

#[test]
fn dropw() {
    let asm_op = "dropw";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_stack(&[5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 0, 0, 0, 0]);
}

#[test]
fn padw() {
    let asm_op = "padw";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_stack(&[0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
}

#[test]
fn dup() {
    let asm_op = "dup";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_stack(&[1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
}

#[test]
fn dupn() {
    let asm_op = "dup.1";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_stack(&[2, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
}

#[test]
fn dupn_fail() {
    let asm_op = "dup.16";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_error(TestError::AssemblyError("parameter"));
}

#[test]
fn dupw() {
    let asm_op = "dupw";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_stack(&[1, 2, 3, 4, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
}

#[test]
fn dupwn() {
    let asm_op = "dupw.1";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_stack(&[5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
}

#[test]
fn dupwn_fail() {
    let asm_op = "dupw.4";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_error(TestError::AssemblyError("parameter"));
}

#[test]
fn swap() {
    let asm_op = "swap";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_stack(&[2, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
}

#[test]
fn swapn() {
    let asm_op = "swap.2";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_stack(&[3, 2, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
}

#[test]
fn swapn_fail() {
    let asm_op = "swap.16";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_error(TestError::AssemblyError("parameter"));
}

#[test]
fn swapw() {
    let asm_op = "swapw";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_stack(&[5, 6, 7, 8, 1, 2, 3, 4, 9, 10, 11, 12, 13, 14, 15, 16]);
}

#[test]
fn swapwn() {
    let asm_op = "swapw.2";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_stack(&[9, 10, 11, 12, 5, 6, 7, 8, 1, 2, 3, 4, 13, 14, 15, 16]);
}

#[test]
fn swapwn_fail() {
    let asm_op = "swapw.4";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_error(TestError::AssemblyError("parameter"));
}

#[test]
fn swapdw() {
    let asm_op = "swapdw";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_stack(&[9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn movup() {
    let asm_op = "movup.2";
    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_stack(&[3, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
}

#[test]
fn movup_fail() {
    let asm_op = "movup.0";
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_error(TestError::AssemblyError("parameter"));

    let asm_op = "movup.1";
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_error(TestError::AssemblyError("parameter"));

    let asm_op = "movup.16";
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_error(TestError::AssemblyError("parameter"));
}

#[test]
fn movupw() {
    let asm_op = "movupw.2";
    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_stack(&[9, 10, 11, 12, 1, 2, 3, 4, 5, 6, 7, 8, 13, 14, 15, 16]);
}

#[test]
fn movupw_fail() {
    let asm_op = "movupw.0";
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_error(TestError::AssemblyError("parameter"));

    let asm_op = "movupw.1";
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_error(TestError::AssemblyError("parameter"));

    let asm_op = "movupw.4";
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_error(TestError::AssemblyError("parameter"));
}

#[test]
fn movdn() {
    let asm_op = "movdn.2";
    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_stack(&[2, 3, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
}

#[test]
fn movdn_fail() {
    let asm_op = "movdn.0";
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_error(TestError::AssemblyError("parameter"));

    let asm_op = "movdn.1";
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_error(TestError::AssemblyError("parameter"));

    let asm_op = "movdn.16";
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_error(TestError::AssemblyError("parameter"));
}

#[test]
fn movdnw() {
    let asm_op = "movdnw.2";
    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_stack(&[5, 6, 7, 8, 9, 10, 11, 12, 1, 2, 3, 4, 13, 14, 15, 16]);
}

#[test]
fn movdnw_fail() {
    let asm_op = "movdnw.0";
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_error(TestError::AssemblyError("parameter"));

    let asm_op = "movdnw.1";
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_error(TestError::AssemblyError("parameter"));

    let asm_op = "movdnw.4";
    let test = build_op_test!(asm_op, &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
    test.expect_error(TestError::AssemblyError("parameter"));
}

#[test]
fn cswap() {
    let asm_op = "cswap";
    // --- simple cases ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
    test.expect_stack(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0]);

    let test = build_op_test!(asm_op, &[15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 1]);
    test.expect_stack(&[2, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0]);
}

#[test]
fn cswapw() {
    let asm_op = "cswapw";
    // --- simple cases ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
    test.expect_stack(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0]);

    let test = build_op_test!(asm_op, &[15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 1]);
    test.expect_stack(&[5, 6, 7, 8, 1, 2, 3, 4, 9, 10, 11, 12, 13, 14, 15, 0]);
}

#[test]
fn cdrop() {
    let asm_op = "cdrop";
    // --- simple cases ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
    test.expect_stack(&[2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 0]);

    let test = build_op_test!(asm_op, &[15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 1]);
    test.expect_stack(&[1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 0]);
}

#[test]
fn cdropw() {
    let asm_op = "cdropw";
    // --- simple cases ----------------------------------------------------------------------------

    let test = build_op_test!(asm_op, &[15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
    test.expect_stack(&[5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 0, 0, 0, 0]);

    let test = build_op_test!(asm_op, &[15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 1]);
    test.expect_stack(&[1, 2, 3, 4, 9, 10, 11, 12, 13, 14, 15, 0, 0, 0, 0, 0]);
}

proptest! {

    #[test]
    fn drop_proptest(test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE)) {
        let asm_op = "drop";
        let mut expected_values = test_values.clone();
        expected_values.remove(STACK_TOP_SIZE - 1);
        expected_values.reverse();
        expected_values.push(0);
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn dropw_proptest(test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE)) {
        let asm_op = "dropw";
        let mut expected_values = test_values.clone();
        expected_values.truncate(STACK_TOP_SIZE - WORD_SIZE);
        expected_values.reverse();
        expected_values.append(&mut vec![0; WORD_SIZE]);
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn padw_proptest(test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE)) {
        let asm_op = "padw";
        let mut expected_values = test_values.clone();
        expected_values.drain(0..WORD_SIZE);
        expected_values.append(&mut vec![0; WORD_SIZE]);
        expected_values.reverse();
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn dup_proptest(test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE)) {
        let asm_op = "dup";
        let mut expected_values = test_values.clone();
        expected_values.remove(0);
        expected_values.push(*expected_values.last().unwrap());
        expected_values.reverse();
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn dupn_proptest(test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE), n in 0_usize..STACK_TOP_SIZE) {
        let asm_op = format!("dup.{n}");
        let mut expected_values = test_values.clone();
        let dup_idx = STACK_TOP_SIZE - n - 1;
        let a = expected_values[dup_idx];
        expected_values.remove(0);
        expected_values.push(a);
        expected_values.reverse();
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn dupw_proptest(test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE)) {
        let asm_op = "dupw";
        let mut expected_values = test_values.clone();
        expected_values.drain(0..WORD_SIZE);
        let dupw_idx = STACK_TOP_SIZE - WORD_SIZE;
        let mut a = test_values[dupw_idx..].to_vec();
        expected_values.append(&mut a);
        expected_values.reverse();
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn dupwn_proptest(test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE), n in 0_usize..WORD_SIZE) {
        let asm_op = format!("dupw.{n}");
        let mut expected_values = test_values.clone();
        expected_values.drain(0..WORD_SIZE);
        let start_dupw_idx = STACK_TOP_SIZE - WORD_SIZE * (n + 1);
        let end_dupw_idx = STACK_TOP_SIZE - WORD_SIZE * n;
        let mut a = test_values[start_dupw_idx..end_dupw_idx].to_vec();
        expected_values.append(&mut a);
        expected_values.reverse();
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn swap_proptest(test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE)) {
        let asm_op = "swap";
        let mut expected_values = test_values.clone();
        expected_values.reverse();
        expected_values.swap(0, 1);
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn swapn_proptest(test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE), n in 1_usize..STACK_TOP_SIZE) {
        let asm_op = format!("swap.{n}");
        let mut expected_values = test_values.clone();
        expected_values.reverse();
        expected_values.swap(0, n);
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn swapw_proptest(test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE)) {
        let asm_op = "swapw";
        let mut expected_values = test_values.clone();
        let mut a = expected_values.split_off(WORD_SIZE * 3);
        let mut b = expected_values.split_off(WORD_SIZE * 2);
        expected_values.append(&mut a);
        expected_values.append(&mut b);
        expected_values.reverse();
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn swapwn_proptest(test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE), n in 1_usize..WORD_SIZE) {
        let asm_op = format!("swapw.{n}");
        let mut expected_values = test_values.clone();
        let start_swapwn_idx = WORD_SIZE * (STACK_TOP_SIZE / WORD_SIZE - n - 1);
        let mut a = expected_values.split_off(start_swapwn_idx);
        let mut b = a.split_off(WORD_SIZE);
        let mut c = b.split_off(b.len() - WORD_SIZE);
        expected_values.append(&mut c);
        expected_values.append(&mut b);
        expected_values.append(&mut a);
        expected_values.reverse();
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn swapdw_proptest(test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE)) {
        let asm_op = "swapdw";
        let mut expected_values = test_values[..(WORD_SIZE * 2)].to_vec();
        let mut b = test_values[(WORD_SIZE * 2)..].to_vec();
        expected_values.reverse();
        b.reverse();
        expected_values.append(&mut b);
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn movup_proptest(test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE), movup_idx in 2_usize..STACK_TOP_SIZE) {
        let asm_op = format!("movup.{movup_idx}");
        let mut expected_values = test_values.clone();
        let idx1 = STACK_TOP_SIZE - movup_idx - 1;
        let movup_value = expected_values[idx1];
        expected_values.remove(idx1);
        expected_values.push(movup_value);
        expected_values.reverse();
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn movupw_proptest(test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE), movupw_idx in 2_usize..WORD_SIZE) {
        let asm_op = format!("movupw.{movupw_idx}");
        let start_movupw_idx = STACK_TOP_SIZE - (movupw_idx + 1) * WORD_SIZE;
        let end_movupw_idx = STACK_TOP_SIZE - movupw_idx * WORD_SIZE;
        let mut movupw_values = test_values[start_movupw_idx..end_movupw_idx].to_vec();
        let mut expected_values = test_values[..start_movupw_idx].to_vec();
        expected_values.append(&mut test_values[end_movupw_idx..].to_vec());
        expected_values.append(&mut movupw_values);
        expected_values.reverse();
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn movdn_proptest(test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE), movdn_idx in 2_usize..STACK_TOP_SIZE) {
        let asm_op = format!("movdn.{movdn_idx}");
        let mut expected_values = test_values.clone();
        let idx1 = STACK_TOP_SIZE - 1;
        let movdn_value = expected_values[idx1];
        expected_values.remove(idx1);
        expected_values.insert(idx1 - movdn_idx, movdn_value);
        expected_values.reverse();
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn movdnw_proptest(test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE), movdnw_idx in 2_usize..WORD_SIZE) {
        let asm_op = format!("movdnw.{movdnw_idx}");
        let idx1 = STACK_TOP_SIZE - (movdnw_idx + 1) * WORD_SIZE;
        let movdnw_idx = STACK_TOP_SIZE - WORD_SIZE;
        let mut movdnw_values = test_values[movdnw_idx..].to_vec();
        let mut expected_values = test_values[..idx1].to_vec();
        expected_values.append(&mut movdnw_values);
        expected_values.append(&mut test_values[idx1..movdnw_idx].to_vec());
        expected_values.reverse();
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn cswap_proptest(mut test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE - 1), c in 0_u64..2) {
        let asm_op = "cswap";
        test_values.push(c);
        let mut expected_values = test_values.clone();
        expected_values.reverse();
        if c == 1 {
            expected_values.swap(1, 2)
        };
        expected_values.remove(0);
        expected_values.push(0);
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn cswapw_proptest(mut test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE - 1), c in 0_u64..2) {
        let asm_op = "cswapw";
        let mut a = test_values.clone();
        a.reverse();
        test_values.push(c);
        let mut expected_values = vec![];
        if c == 1 {
            expected_values.append(&mut a[WORD_SIZE..(WORD_SIZE * 2)].to_vec());
            expected_values.append(&mut a[..WORD_SIZE].to_vec());
            expected_values.append(&mut a[(WORD_SIZE * 2)..].to_vec());
        } else {
            expected_values = a;
        }
        expected_values.push(0);
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn cdrop_proptest(mut test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE - 1), c in 0_u64..2) {
        let asm_op = "cdrop";
        test_values.push(c);
        let mut expected_values = test_values.clone();
        expected_values.reverse();
        if c == 1 {
            expected_values.remove(2)
        } else {
            expected_values.remove(1)
        };
        expected_values.remove(0);
        expected_values.push(0);
        expected_values.push(0);
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

    #[test]
    fn cdropw_proptest(mut test_values in prop::collection::vec(any::<u64>(), STACK_TOP_SIZE - 1), c in 0_u64..2) {
        let asm_op = "cdropw";
        let mut a = test_values.clone();
        a.reverse();
        test_values.push(c);
        let mut expected_values = a.clone();
        if c == 0 {
            expected_values.drain(0..WORD_SIZE);
            expected_values.append(&mut vec![0; WORD_SIZE]);
            expected_values.push(0);
        } else {
            expected_values = a[..WORD_SIZE].to_vec();
            a.drain(0..(WORD_SIZE * 2));
            expected_values.append(&mut a.to_vec());
            expected_values.append(&mut vec![0; WORD_SIZE]);
            expected_values.push(0);
        }
        build_op_test!(asm_op, &test_values).prop_expect_stack(&expected_values)?;
    }

}
