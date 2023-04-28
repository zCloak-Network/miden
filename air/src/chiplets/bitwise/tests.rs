use super::{
    enforce_constraints, get_periodic_values, EvaluationFrame, BITWISE_A_COL_IDX,
    BITWISE_A_COL_RANGE, BITWISE_B_COL_IDX, BITWISE_B_COL_RANGE, BITWISE_OUTPUT_COL_IDX,
    BITWISE_PREV_OUTPUT_COL_IDX, BITWISE_SELECTOR_COL_IDX, NUM_CONSTRAINTS, NUM_DECOMP_BITS,
    OP_CYCLE_LEN,
};
use rand_utils::rand_value;
use vm_core::{
    chiplets::{
        bitwise::{BITWISE_AND, BITWISE_XOR},
        BITWISE_TRACE_RANGE,
    },
    Felt, FieldElement, TRACE_WIDTH,
};

use proptest::prelude::*;

// UNIT TESTS
// ================================================================================================

/// Tests that the bitwise constraints do not all evaluate to zero if the internal selector which
/// specify the operation change within a cycle.
#[test]
fn test_bitwise_change_ops_fail() {
    let expected = [Felt::ZERO; NUM_CONSTRAINTS];

    let a = rand_value::<u32>();
    let b = rand_value::<u32>();
    let cycle_row: usize = rand_value::<u8>() as usize % (OP_CYCLE_LEN - 1);

    let frame = get_test_frame_with_two_ops(BITWISE_XOR, BITWISE_AND, a, b, cycle_row);
    let result = get_constraint_evaluation(frame, cycle_row);

    // The selector flag changes, so that constraint should fail.
    assert_ne!(result[1], expected[1]);
    // All other constraints should evaluate to zero.
    assert_eq!(result[0..1], expected[0..1]);
    assert_eq!(result[2..], expected[2..]);
}

/// Tests that the prover cannot set an incorrect output during BITWISE_AND at the first row in the
/// cycle when the low limb of a is one.
#[test]
fn output_aggregation_and() {
    let cycle_row = 0;

    // create a valid test frame manually
    let mut current = vec![Felt::ZERO; TRACE_WIDTH];
    let mut next = vec![Felt::ZERO; TRACE_WIDTH];

    let current_bitwise = [
        // selector
        BITWISE_AND,
        // a
        Felt::ONE,
        // b
        Felt::new(9),
        // decomposition of a
        Felt::ONE,
        Felt::ZERO,
        Felt::ZERO,
        Felt::ZERO,
        // decomposition of b
        Felt::ONE,
        Felt::ZERO,
        Felt::ZERO,
        Felt::ONE,
        // previous output
        Felt::ZERO,
        // assert a false output
        Felt::new(1337),
    ];

    let next_bitwise = [
        // selector
        BITWISE_AND,
        // a
        Felt::new(19),
        // b
        Felt::new(157),
        // decomposition of a
        Felt::ONE,
        Felt::ONE,
        Felt::ZERO,
        Felt::ZERO,
        // decomposition of b
        Felt::ONE,
        Felt::ZERO,
        Felt::ONE,
        Felt::ONE,
        // previous output
        Felt::new(1337),
        // output
        Felt::new(21393),
    ];

    current[BITWISE_TRACE_RANGE].copy_from_slice(&current_bitwise);
    next[BITWISE_TRACE_RANGE].copy_from_slice(&next_bitwise);

    let frame = EvaluationFrame::<Felt>::from_rows(current, next);
    let result = get_constraint_evaluation(frame, cycle_row);

    // expect a failure for the output aggregation constraint (the last one)
    assert_ne!(Felt::ZERO, result[NUM_CONSTRAINTS - 1]);
}

// RANDOMIZED TESTS
// ================================================================================================

proptest! {
    /// Tests that the bitwise constraints evaluate to zero on valid frames within a cycle which
    /// compute the bitwise AND operation.
    #[test]
    fn test_bitwise_and(a in any::<u32>(), b in any::<u32>(), cycle_row in 0..(OP_CYCLE_LEN - 1)) {
        let expected = [Felt::ZERO; NUM_CONSTRAINTS];
        let frame = get_test_frame(BITWISE_AND, a, b, cycle_row);
        let result = get_constraint_evaluation(frame, cycle_row);
        assert_eq!(expected, result);
    }

    /// Tests that the bitwise constraints evaluate to zero on valid frames within a cycle which
    /// compute the bitwise XOR operation.
    #[test]
    fn test_bitwise_xor(a in any::<u32>(), b in any::<u32>(), cycle_row in 0..(OP_CYCLE_LEN - 1)) {
        let expected = [Felt::ZERO; NUM_CONSTRAINTS];
        let frame = get_test_frame(BITWISE_XOR, a, b, cycle_row);
        let result = get_constraint_evaluation(frame, cycle_row);
        assert_eq!(expected, result);
    }
}

// TEST HELPERS
// ================================================================================================

/// Returns the result of Bitwise constraint evaluations on the provided frame starting at the
/// specified row.
fn get_constraint_evaluation(frame: EvaluationFrame<Felt>, row: usize) -> [Felt; NUM_CONSTRAINTS] {
    let periodic_values = get_periodic_values(row);
    let mut result = [Felt::ZERO; NUM_CONSTRAINTS];

    enforce_constraints(&frame, &periodic_values, &mut result, Felt::ONE);

    result
}

/// Generates the correct current and next rows for the specified operation, inputs, and current
/// cycle row number and returns an EvaluationFrame for testing. It only tests frames within a
/// cycle.
///
/// # Errors
/// It expects the specified `cycle_row_num` for the current row to be such that the next row will
/// still be in the same cycle. It will fail if the row number input is >= OP_CYCLE_LEN - 1.
pub fn get_test_frame(
    operation: Felt,
    a: u32,
    b: u32,
    cycle_row_num: usize,
) -> EvaluationFrame<Felt> {
    assert!(
        cycle_row_num < OP_CYCLE_LEN - 1,
        "Failed to build test EvaluationFrame for bitwise operation. The next row would be in a new cycle."
    );

    // Initialize the rows.
    let mut current = vec![Felt::ZERO; TRACE_WIDTH];
    let mut next = vec![Felt::ZERO; TRACE_WIDTH];

    // Set the operation selectors.
    current[BITWISE_SELECTOR_COL_IDX] = operation;
    next[BITWISE_SELECTOR_COL_IDX] = operation;

    // Set the input aggregation and decomposition values.
    set_frame_inputs(&mut current, &mut next, a, b, cycle_row_num);

    // Compute the output for the specified operation and inputs and shift it for each row.
    let (previous_shift, current_shift, next_shift) = get_row_shifts(cycle_row_num);
    let result = get_output(operation, a, b);
    let output_current = result >> current_shift;
    let output_next = result >> next_shift;

    // Set the previous output.
    let output_prev = if cycle_row_num == 0 {
        Felt::ZERO
    } else {
        Felt::new((result >> previous_shift) as u64)
    };
    current[BITWISE_PREV_OUTPUT_COL_IDX] = output_prev;
    next[BITWISE_PREV_OUTPUT_COL_IDX] = Felt::new((output_current) as u64);

    // Set the output.
    current[BITWISE_OUTPUT_COL_IDX] = Felt::new(output_current as u64);
    next[BITWISE_OUTPUT_COL_IDX] = Felt::new((output_next) as u64);

    EvaluationFrame::<Felt>::from_rows(current, next)
}

/// Generates the current and next rows for the provided inputs, current cycle row number, and the
/// operations specified for each row, and returns an EvaluationFrame for testing. It only tests
/// frames within a cycle.
///
/// # Errors
/// It expects the specified `cycle_row_num` for the current row to be such that the next row will
/// still be in the same cycle. It will fail if the row number input is >= OP_CYCLE_LEN - 1.
pub fn get_test_frame_with_two_ops(
    op_current: Felt,
    op_next: Felt,
    a: u32,
    b: u32,
    cycle_row_num: usize,
) -> EvaluationFrame<Felt> {
    assert!(
        cycle_row_num < OP_CYCLE_LEN - 1,
        "Failed to build test EvaluationFrame for bitwise operation. The next row would be in a new cycle."
    );

    // Initialize the rows.
    let mut current = vec![Felt::ZERO; TRACE_WIDTH];
    let mut next = vec![Felt::ZERO; TRACE_WIDTH];

    // Set the operation selector.
    current[BITWISE_SELECTOR_COL_IDX] = op_current;
    next[BITWISE_SELECTOR_COL_IDX] = op_next;

    // Set the input aggregation and decomposition values.
    set_frame_inputs(&mut current, &mut next, a, b, cycle_row_num);

    // Compute the outputs for the specified operations and inputs and shift them for each row.
    let (previous_shift, current_shift, next_shift) = get_row_shifts(cycle_row_num);
    let result_op_current = get_output(op_current, a, b);
    let output_current = result_op_current >> current_shift;
    let output_next = get_output(op_next, a, b) >> next_shift;

    // Set the previous output.
    let output_prev = if cycle_row_num == 0 {
        Felt::ZERO
    } else {
        Felt::new((result_op_current >> previous_shift) as u64)
    };
    current[BITWISE_PREV_OUTPUT_COL_IDX] = output_prev;
    next[BITWISE_PREV_OUTPUT_COL_IDX] = Felt::new((output_current) as u64);

    // Set the output.
    current[BITWISE_OUTPUT_COL_IDX] = Felt::new(output_current as u64);
    next[BITWISE_OUTPUT_COL_IDX] = Felt::new((output_next) as u64);

    EvaluationFrame::<Felt>::from_rows(current, next)
}

/// Returns the shift amount for the previous, current, and next rows, based on the `cycle_row_num`,
/// which is the number of the `current` row within the operation cycle.
fn get_row_shifts(cycle_row_num: usize) -> (usize, usize, usize) {
    // Define the shift amount for output in this row and the next row.
    let current_shift = NUM_DECOMP_BITS * (OP_CYCLE_LEN - cycle_row_num - 1);
    let previous_shift = current_shift + NUM_DECOMP_BITS;
    let next_shift = current_shift - NUM_DECOMP_BITS;

    (previous_shift, current_shift, next_shift)
}

/// Sets the input aggregation and decomposition columns in the provided current and next rows with
/// the correct values corresponding to the provided inputs `a` and `b` and the specified
/// `cycle_row_num`, which is the number of the `current` row within the operation cycle.
fn set_frame_inputs(current: &mut [Felt], next: &mut [Felt], a: u32, b: u32, cycle_row_num: usize) {
    // Get the shift amounts for the specified rows.
    let (_, current_shift, next_shift) = get_row_shifts(cycle_row_num);

    // Set the input aggregation values.
    let current_a = (a >> current_shift) as u64;
    let current_b = (b >> current_shift) as u64;
    let next_a = (a >> next_shift) as u64;
    let next_b = (b >> next_shift) as u64;

    current[BITWISE_A_COL_IDX] = Felt::new(current_a);
    next[BITWISE_A_COL_IDX] = Felt::new(next_a);
    current[BITWISE_B_COL_IDX] = Felt::new(current_b);
    next[BITWISE_B_COL_IDX] = Felt::new(next_b);

    // Set the input decomposition values.
    for idx in 0..NUM_DECOMP_BITS {
        current[BITWISE_A_COL_RANGE.start + idx] = Felt::new((current_a >> idx) & 1);
        current[BITWISE_B_COL_RANGE.start + idx] = Felt::new((current_b >> idx) & 1);
        next[BITWISE_A_COL_RANGE.start + idx] = Felt::new((next_a >> idx) & 1);
        next[BITWISE_B_COL_RANGE.start + idx] = Felt::new((next_b >> idx) & 1);
    }
}

/// Returns the final output result of applying the specified operation to the provided inputs.
fn get_output(operation: Felt, a: u32, b: u32) -> u32 {
    if operation == BITWISE_AND {
        a & b
    } else if operation == BITWISE_XOR {
        a ^ b
    } else {
        panic!("Test bitwise EvaluationFrame requested for unrecognized operation.");
    }
}
