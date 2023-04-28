use super::{
    Assertion, AuxTraceRandElements, EvaluationFrame, Felt, FieldElement,
    TransitionConstraintDegree, ONE, STACK_TRACE_OFFSET, ZERO,
};
use crate::utils::{are_equal, is_binary};
use vm_core::{
    decoder::{IS_CALL_FLAG_COL_IDX, USER_OP_HELPERS_OFFSET},
    stack::STACK_TOP_SIZE,
    utils::collections::Vec,
    StackOutputs, StarkField, CLK_COL_IDX, DECODER_TRACE_OFFSET, FMP_COL_IDX,
    STACK_AUX_TRACE_OFFSET,
};

pub mod field_ops;
pub mod io_ops;
pub mod op_flags;
pub mod overflow;
pub mod stack_manipulation;
pub mod system_ops;
pub mod u32_ops;

// CONSTANTS
// ================================================================================================

const B0_COL_IDX: usize = STACK_TRACE_OFFSET + STACK_TOP_SIZE;
const B1_COL_IDX: usize = B0_COL_IDX + 1;
const H0_COL_IDX: usize = B1_COL_IDX + 1;

// --- Main constraints ---------------------------------------------------------------------------

/// The number of boundary constraints required by the Stack, which is all stack positions for
/// inputs and outputs as well as the initial values of the bookkeeping columns.
pub const NUM_ASSERTIONS: usize = 2 * STACK_TOP_SIZE + 2;

/// The number of general constraints in the stack operations.
pub const NUM_GENERAL_CONSTRAINTS: usize = 17;

/// The degrees of constraints in the general stack operations. Each operation being executed
/// either shifts the stack to the left, right or doesn't effect it at all. Therefore, majority
/// of the general transtitions of a stack item would be common across the operations and composite
/// flags were introduced to compute the individual stack item transition. A particular item lets say
/// at depth ith in the next stack frame can be transitioned into from ith depth (no shift op) or
/// (i+1)th depth(left shift) or (i-1)th depth(right shift) in the current frame. Therefore, the VM
/// would require only 16 general constraints to encompass all the 16 stack positions.
/// The last constraint checks if the top element in the stack is a binary or not.
pub const CONSTRAINT_DEGREES: [usize; NUM_GENERAL_CONSTRAINTS] = [
    // Each degree are being multiplied with the respective composite flags which are of degree 7.
    // Therefore, all the degree would incorporate 7 in their degree calculation.
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 9,
];

// --- Auxiliary column constraints ---------------------------------------------------------------

/// The number of auxiliary assertions.
pub const NUM_AUX_ASSERTIONS: usize = 2;

// STACK OPERATIONS TRANSITION CONSTRAINTS
// ================================================================================================

/// Build the transition constraint degrees for the stack module and all the stack operations.
pub fn get_transition_constraint_degrees() -> Vec<TransitionConstraintDegree> {
    let mut degrees = overflow::get_transition_constraint_degrees();
    // system operations constraints degrees.
    degrees.append(&mut system_ops::get_transition_constraint_degrees());
    // field operations constraints degrees.
    degrees.append(&mut field_ops::get_transition_constraint_degrees());
    // stack manipulation operations constraints degrees.
    degrees.append(&mut stack_manipulation::get_transition_constraint_degrees());
    // u32 operations constraints degrees.
    degrees.append(&mut u32_ops::get_transition_constraint_degrees());
    // input/output operations constraints degrees.
    degrees.append(&mut io_ops::get_transition_constraint_degrees());
    // Add the degrees of general constraints.
    degrees.append(
        &mut CONSTRAINT_DEGREES
            .iter()
            .map(|&degree| TransitionConstraintDegree::new(degree))
            .collect(),
    );

    degrees
}

/// Returns the number of transition constraints for the stack operations.
pub fn get_transition_constraint_count() -> usize {
    overflow::get_transition_constraint_count()
        + system_ops::get_transition_constraint_count()
        + field_ops::get_transition_constraint_count()
        + stack_manipulation::get_transition_constraint_count()
        + u32_ops::get_transition_constraint_count()
        + io_ops::get_transition_constraint_count()
        + NUM_GENERAL_CONSTRAINTS
}

/// Enforces constraints for the stack module and all stack operations.
pub fn enforce_constraints<E: FieldElement<BaseField = Felt>>(
    frame: &EvaluationFrame<E>,
    result: &mut [E],
) -> usize {
    let mut index = 0;

    let op_flag = op_flags::OpFlags::new(frame);

    // Enforces stack operations unique constraints.
    index += enforce_unique_constraints(frame, result, &op_flag);

    // Enforces stack operations general constraints.
    index += enforce_general_constraints(frame, &mut result[index..], &op_flag);

    index
}

/// Enforces unique constraints of all the stack ops.
pub fn enforce_unique_constraints<E: FieldElement>(
    frame: &EvaluationFrame<E>,
    result: &mut [E],
    op_flag: &op_flags::OpFlags<E>,
) -> usize {
    let mut constraint_offset = 0;

    overflow::enforce_constraints(frame, result, op_flag);
    constraint_offset += overflow::get_transition_constraint_count();

    // system operations transition constraints.
    system_ops::enforce_constraints(frame, &mut result[constraint_offset..], op_flag);
    constraint_offset += system_ops::get_transition_constraint_count();

    // field operations transition constraints.
    field_ops::enforce_constraints(frame, &mut result[constraint_offset..], op_flag);
    constraint_offset += field_ops::get_transition_constraint_count();

    // stack manipulation operations transition constraints.
    stack_manipulation::enforce_constraints(frame, &mut result[constraint_offset..], op_flag);
    constraint_offset += stack_manipulation::get_transition_constraint_count();

    // u32 operations transition constraints.
    u32_ops::enforce_constraints(frame, &mut result[constraint_offset..], op_flag);
    constraint_offset += u32_ops::get_transition_constraint_count();

    // input/output operations transition constraints.
    io_ops::enforce_constraints(frame, &mut result[constraint_offset..], op_flag);
    constraint_offset += io_ops::get_transition_constraint_count();

    constraint_offset
}

/// Enforces general constraints of all the stack ops.
pub fn enforce_general_constraints<E: FieldElement>(
    frame: &EvaluationFrame<E>,
    result: &mut [E],
    op_flag: &op_flags::OpFlags<E>,
) -> usize {
    // enforces constraint on the 1st element in the stack in the next trace.
    let flag_sum = op_flag.no_shift_at(0) + op_flag.left_shift_at(1);
    let expected_next_item = op_flag.no_shift_at(0) * frame.stack_item(0)
        + op_flag.left_shift_at(1) * frame.stack_item(1);
    result[0] = are_equal(frame.stack_item_next(0) * flag_sum, expected_next_item);

    // enforces constraint on the ith element in the stack in the next trace.
    #[allow(clippy::needless_range_loop)]
    for i in 1..NUM_GENERAL_CONSTRAINTS - 2 {
        let flag_sum =
            op_flag.no_shift_at(i) + op_flag.left_shift_at(i + 1) + op_flag.right_shift_at(i - 1);
        let expected_next_item = op_flag.no_shift_at(i) * frame.stack_item(i)
            + op_flag.left_shift_at(i + 1) * frame.stack_item(i + 1)
            + op_flag.right_shift_at(i - 1) * frame.stack_item(i - 1);

        result[i] = are_equal(frame.stack_item_next(i) * flag_sum, expected_next_item);
    }

    // enforces constraint on the last element in the stack in the next trace.
    let flag_sum = op_flag.no_shift_at(15) + op_flag.right_shift_at(14);
    let expected_next_item = op_flag.no_shift_at(15) * frame.stack_item(15)
        + op_flag.right_shift_at(14) * frame.stack_item(14);
    result[NUM_GENERAL_CONSTRAINTS - 2] = are_equal(
        frame.stack_item_next(NUM_GENERAL_CONSTRAINTS - 2) * flag_sum,
        expected_next_item,
    );

    // enforces constraint on the top element being binary or not.
    let top_binary_flag = op_flag.top_binary();
    result[NUM_GENERAL_CONSTRAINTS - 1] = top_binary_flag * is_binary(frame.stack_item(0));

    NUM_GENERAL_CONSTRAINTS
}
// BOUNDARY CONSTRAINTS
// ================================================================================================

// --- MAIN TRACE ---------------------------------------------------------------------------------

/// Returns the stack's boundary assertions for the main trace at the first step.
pub fn get_assertions_first_step(result: &mut Vec<Assertion<Felt>>, stack_inputs: &[Felt]) {
    // stack columns at the first step should be set to stack inputs, excluding overflow inputs.
    for (i, &value) in stack_inputs.iter().take(STACK_TOP_SIZE).enumerate() {
        result.push(Assertion::single(STACK_TRACE_OFFSET + i, 0, value));
    }

    // if there are remaining slots on top of the stack without specified values, set them to ZERO.
    for i in stack_inputs.len()..STACK_TOP_SIZE {
        result.push(Assertion::single(STACK_TRACE_OFFSET + i, 0, ZERO));
    }

    // get the initial values for the bookkeeping columns.
    let mut depth = STACK_TOP_SIZE;
    let mut overflow_addr = ZERO;
    if stack_inputs.len() > STACK_TOP_SIZE {
        depth = stack_inputs.len();
        overflow_addr = -ONE;
    }

    // b0 should be initialized to the depth of the stack.
    result.push(Assertion::single(B0_COL_IDX, 0, Felt::new(depth as u64)));

    // b1 should be initialized to the address of the last row in the overflow table, which is 0
    // when the overflow table is empty and -1 mod p when the stack is initialized with overflow.
    result.push(Assertion::single(B1_COL_IDX, 0, overflow_addr));
}

/// Returns the stack's boundary assertions for the main trace at the last step.
pub fn get_assertions_last_step(
    result: &mut Vec<Assertion<Felt>>,
    step: usize,
    stack_outputs: &StackOutputs,
) {
    // stack columns at the last step should be set to stack outputs, excluding overflow outputs
    for (i, value) in stack_outputs.stack_top().iter().enumerate() {
        result.push(Assertion::single(STACK_TRACE_OFFSET + i, step, *value));
    }
}

// --- AUXILIARY COLUMNS --------------------------------------------------------------------------

/// Returns the stack's boundary assertions for auxiliary columns at the first step.
pub fn get_aux_assertions_first_step<E: FieldElement>(
    result: &mut Vec<Assertion<E>>,
    alphas: &AuxTraceRandElements<E>,
    stack_inputs: &[Felt],
) where
    E: FieldElement<BaseField = Felt>,
{
    let step = 0;
    let value = if stack_inputs.len() > STACK_TOP_SIZE {
        get_overflow_table_init(alphas.get_segment_elements(0), &stack_inputs[STACK_TOP_SIZE..])
    } else {
        E::ONE
    };

    result.push(Assertion::single(STACK_AUX_TRACE_OFFSET, step, value));
}

/// Returns the stack's boundary assertions for auxiliary columns at the last step.
pub fn get_aux_assertions_last_step<E: FieldElement>(
    result: &mut Vec<Assertion<E>>,
    alphas: &AuxTraceRandElements<E>,
    stack_outputs: &StackOutputs,
    step: usize,
) where
    E: FieldElement<BaseField = Felt>,
{
    let value = if stack_outputs.has_overflow() {
        get_overflow_table_final(alphas.get_segment_elements(0), stack_outputs)
    } else {
        E::ONE
    };

    result.push(Assertion::single(STACK_AUX_TRACE_OFFSET, step, value));
}

// BOUNDARY CONSTRAINT HELPERS
// ================================================================================================

// --- AUX TRACE ----------------------------------------------------------------------------------

/// Gets the initial value of the overflow table auxiliary column from the provided sets of initial
/// values and random elements.
fn get_overflow_table_init<E: FieldElement>(alphas: &[E], init_values: &[Felt]) -> E
where
    E: FieldElement<BaseField = Felt>,
{
    let mut value = E::ONE;
    let mut prev_clk = ZERO;
    let mut clk = Felt::from(Felt::MODULUS - init_values.len() as u64);

    // the values are in the overflow table in reverse order, since the deepest stack
    // value is added to the overflow table first.
    for &input in init_values.iter().rev() {
        value *= alphas[0]
            + alphas[1].mul_base(clk)
            + alphas[2].mul_base(input)
            + alphas[3].mul_base(prev_clk);
        prev_clk = clk;
        clk += ONE;
    }

    value
}

/// Gets the final value of the overflow table auxiliary column from the provided program outputs
/// and random elements.
fn get_overflow_table_final<E: FieldElement>(alphas: &[E], stack_outputs: &StackOutputs) -> E
where
    E: FieldElement<BaseField = Felt>,
{
    let mut value = E::ONE;

    // When the overflow table is non-empty, we expect at least 2 addresses (the `prev` value of
    // the first row and the address value(s) of the row(s)) and more than STACK_TOP_SIZE
    // elements in the stack.
    let mut prev = stack_outputs.overflow_prev();
    for (clk, val) in stack_outputs.stack_overflow() {
        value *= alphas[0]
            + alphas[1].mul_base(clk)
            + alphas[2].mul_base(val)
            + alphas[3].mul_base(prev);

        prev = clk;
    }

    value
}

// STACK OPERATION EXTENSION TRAIT
// ================================================================================================

trait EvaluationFrameExt<E: FieldElement> {
    // --- Column accessors -----------------------------------------------------------------------

    /// Returns the current value at the specified index in the stack.
    fn stack_item(&self, index: usize) -> E;

    /// Returns the next value at the specified index in the stack.
    fn stack_item_next(&self, index: usize) -> E;

    /// Gets the depth of the stack at the current step.
    fn stack_depth(&self) -> E;

    /// Gets the depth of the stack at the next step.
    fn stack_depth_next(&self) -> E;

    /// Returns the value of the bookkeeping column `b1` at the next step.
    fn stack_overflow_addr_next(&self) -> E;

    /// Returns the current value of stack helper column `h0`.
    fn stack_helper(&self) -> E;

    /// Gets the current element of the clk register in the trace.
    fn clk(&self) -> E;

    /// Gets the next element of the clk register in the trace.
    fn clk_next(&self) -> E;

    /// Gets the current element of the fmp register in the trace.
    fn fmp(&self) -> E;

    /// Gets the next element of the fmp register in the trace.
    fn fmp_next(&self) -> E;

    /// Gets the current value of user op helper register located at the specified index.
    fn user_op_helper(&self, index: usize) -> E;

    /// Returns the value if the `h6` helper register in the decoder which is set to ONE if the
    /// ending block is a `CALL` block.
    fn is_call_end(&self) -> E;
}

impl<E: FieldElement> EvaluationFrameExt<E> for &EvaluationFrame<E> {
    // --- Column accessors -----------------------------------------------------------------------

    #[inline(always)]
    fn stack_item(&self, index: usize) -> E {
        debug_assert!(index < 16, "stack index cannot exceed 15");
        self.current()[STACK_TRACE_OFFSET + index]
    }

    #[inline(always)]
    fn stack_item_next(&self, index: usize) -> E {
        debug_assert!(index < 16, "stack index cannot exceed 15");
        self.next()[STACK_TRACE_OFFSET + index]
    }

    #[inline(always)]
    fn stack_depth(&self) -> E {
        self.current()[B0_COL_IDX]
    }

    #[inline(always)]
    fn stack_depth_next(&self) -> E {
        self.next()[B0_COL_IDX]
    }

    #[inline(always)]
    fn stack_overflow_addr_next(&self) -> E {
        self.next()[B1_COL_IDX]
    }

    #[inline(always)]
    fn stack_helper(&self) -> E {
        self.current()[H0_COL_IDX]
    }

    #[inline(always)]
    fn clk(&self) -> E {
        self.current()[CLK_COL_IDX]
    }

    #[inline(always)]
    fn clk_next(&self) -> E {
        self.next()[CLK_COL_IDX]
    }

    #[inline(always)]
    fn fmp(&self) -> E {
        self.current()[FMP_COL_IDX]
    }

    #[inline(always)]
    fn fmp_next(&self) -> E {
        self.next()[FMP_COL_IDX]
    }

    #[inline(always)]
    fn user_op_helper(&self, index: usize) -> E {
        self.current()[DECODER_TRACE_OFFSET + USER_OP_HELPERS_OFFSET + index]
    }

    #[inline]
    fn is_call_end(&self) -> E {
        self.current()[DECODER_TRACE_OFFSET + IS_CALL_FLAG_COL_IDX]
    }
}
