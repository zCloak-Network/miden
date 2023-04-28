use super::{enforce_constraints, EvaluationFrame, NUM_CONSTRAINTS};
use crate::stack::{
    op_flags::{generate_evaluation_frame, OpFlags},
    B0_COL_IDX,
};
use rand_utils::rand_value;
use vm_core::{Felt, FieldElement, Operation, STACK_TRACE_OFFSET};

// UNIT TESTS
// ================================================================================================

#[test]
fn test_sdepth_operation() {
    let expected = [Felt::ZERO; NUM_CONSTRAINTS];
    let depth = rand_value::<u32>() as u64;

    let frame = get_sdepth_test_frame(depth);
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

/// Generates the correct current and next rows for the SDEPTH operation and inputs and
/// returns an EvaluationFrame for testing.
pub fn get_sdepth_test_frame(depth: u64) -> EvaluationFrame<Felt> {
    // frame initialised with a u32split operation using it's unique opcode.
    let mut frame = generate_evaluation_frame(Operation::SDepth.op_code() as usize);

    // Set the output. The depth of the stack in the current trace should be the top
    // element in the next frame.
    frame.current_mut()[B0_COL_IDX] = Felt::new(depth);
    frame.next_mut()[STACK_TRACE_OFFSET] = Felt::new(depth);

    frame
}
