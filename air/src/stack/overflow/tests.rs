use super::{enforce_constraints, EvaluationFrame, NUM_CONSTRAINTS};
use crate::stack::{
    op_flags::{generate_evaluation_frame, OpFlags},
    B0_COL_IDX, B1_COL_IDX, H0_COL_IDX,
};
use rand_utils::rand_value;
use vm_core::{
    decoder::IS_CALL_FLAG_COL_IDX, Felt, FieldElement, Operation, CLK_COL_IDX,
    DECODER_TRACE_OFFSET, ONE, STACK_TRACE_OFFSET, ZERO,
};

// UNIT TESTS
// ================================================================================================

#[test]
fn test_stack_overflow_constraints() {
    let expected = [Felt::ZERO; NUM_CONSTRAINTS];

    // ------------------ right shift operation ----------------------------------------------------

    let depth = 16 + rand_value::<u32>() as u64;
    let mut frame = generate_evaluation_frame(Operation::Pad.op_code().into());

    // Set the output. The top element in the next frame should be 0.
    frame.current_mut()[CLK_COL_IDX] = Felt::new(8);
    frame.current_mut()[B0_COL_IDX] = Felt::new(depth);
    frame.current_mut()[B1_COL_IDX] = Felt::new(7);
    frame.current_mut()[H0_COL_IDX] = Felt::new(depth - 16).inv();

    frame.next_mut()[B0_COL_IDX] = Felt::new(depth + 1);
    frame.next_mut()[B1_COL_IDX] = frame.current()[CLK_COL_IDX];
    frame.next_mut()[H0_COL_IDX] = Felt::new(depth + 1 - 16).inv();
    frame.next_mut()[CLK_COL_IDX] = Felt::new(9);

    let result = get_constraint_evaluation(frame);
    assert_eq!(expected, result);

    // ------------------ left shift operation- depth 16 ----------------------------------------------------

    let depth = 16;
    let mut frame = generate_evaluation_frame(Operation::Drop.op_code().into());

    // Set the output.
    frame.current_mut()[CLK_COL_IDX] = Felt::new(15);
    frame.current_mut()[B0_COL_IDX] = Felt::new(depth);
    frame.current_mut()[STACK_TRACE_OFFSET + 15] = ONE;

    frame.next_mut()[STACK_TRACE_OFFSET + 14] = ONE;
    frame.current_mut()[STACK_TRACE_OFFSET + 15] = ZERO;
    frame.next_mut()[B0_COL_IDX] = Felt::new(depth);
    frame.next_mut()[B1_COL_IDX] = ZERO;
    frame.next_mut()[CLK_COL_IDX] = Felt::new(16);

    let result = get_constraint_evaluation(frame);
    assert_eq!(expected, result);

    // ------------------ left shift operation- depth 17 ----------------------------------------------------

    let depth = 17;
    let mut frame = generate_evaluation_frame(Operation::Drop.op_code().into());

    // Set the output.
    frame.current_mut()[CLK_COL_IDX] = Felt::new(15);
    frame.current_mut()[B0_COL_IDX] = Felt::new(depth);
    frame.current_mut()[B1_COL_IDX] = Felt::new(12);
    frame.current_mut()[H0_COL_IDX] = ONE;

    frame.next_mut()[B0_COL_IDX] = Felt::new(depth - 1);
    frame.next_mut()[B1_COL_IDX] = ZERO;
    frame.next_mut()[H0_COL_IDX] = ZERO;
    frame.next_mut()[CLK_COL_IDX] = Felt::new(16);

    let result = get_constraint_evaluation(frame);
    assert_eq!(expected, result);

    // ------------------ no shift operation ----------------------------------------------------

    let depth = 16 + rand_value::<u32>() as u64;
    let b1 = rand_value::<u64>();
    let h1 = Felt::new(depth - 16).inv();
    let mut frame = generate_evaluation_frame(Operation::Noop.op_code().into());

    // Set the output.
    frame.current_mut()[CLK_COL_IDX] = ZERO;
    frame.current_mut()[B0_COL_IDX] = Felt::new(depth);
    frame.current_mut()[B1_COL_IDX] = Felt::new(b1);
    frame.current_mut()[H0_COL_IDX] = h1;

    frame.next_mut()[CLK_COL_IDX] = ONE;
    frame.next_mut()[B0_COL_IDX] = Felt::new(depth);
    frame.next_mut()[B1_COL_IDX] = Felt::new(b1);
    frame.next_mut()[H0_COL_IDX] = h1;

    let result = get_constraint_evaluation(frame);
    assert_eq!(expected, result);
}

#[test]
fn test_stack_depth_air() {
    let depth = 16 + rand_value::<u32>() as u64;
    // block with a control block opcode.
    let mut frame = generate_evaluation_frame(Operation::Split.op_code().into());

    // At the start of a control block, the second part of the hasher state gets populated with h2. Therefore, the 7th
    // hasher element alone can't be used as a flag if it's the end of a call block, and the stack depth air constraint
    // will fail in all control flow operations, which shifts the stack either to the right or left.
    frame.current_mut()[CLK_COL_IDX] = ZERO;
    frame.current_mut()[B0_COL_IDX] = Felt::new(depth);
    frame.current_mut()[STACK_TRACE_OFFSET] = ONE;
    // setting it to any u64 random value other than 0.
    frame.current_mut()[DECODER_TRACE_OFFSET + IS_CALL_FLAG_COL_IDX] =
        Felt::new(rand_value::<u32>() as u64);
    frame.current_mut()[B1_COL_IDX] = Felt::new(12);
    frame.current_mut()[H0_COL_IDX] = Felt::new(depth - 16).inv();

    frame.next_mut()[CLK_COL_IDX] = ONE;
    frame.next_mut()[B0_COL_IDX] = Felt::new(depth - 1);
    frame.next_mut()[B1_COL_IDX] = Felt::new(12);
    frame.next_mut()[H0_COL_IDX] = Felt::new(depth - 1 - 16).inv();

    let expected = [Felt::ZERO; NUM_CONSTRAINTS];
    let result = get_constraint_evaluation(frame);

    assert_eq!(expected, result);
}

// TEST HELPERS
// ================================================================================================

/// Returns the result of stack operation constraint evaluations on the provided frame.
fn get_constraint_evaluation(frame: EvaluationFrame<Felt>) -> [Felt; NUM_CONSTRAINTS] {
    let mut result = [Felt::ZERO; NUM_CONSTRAINTS];

    let op_flag = &OpFlags::new(&frame);

    enforce_constraints(&frame, &mut result, op_flag);

    result
}
