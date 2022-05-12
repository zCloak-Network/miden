use super::{super::DecoderTrace, Felt, FieldElement, Operation, NUM_OP_BITS};
use crate::{trace::NUM_RAND_ROWS, ExecutionTrace, Process, ProgramInputs};
use vm_core::{program::blocks::CodeBlock, StarkField, DECODER_TRACE_RANGE};

// CONSTANTS
// ================================================================================================

/// TODO: move to core?
const OP_BITS_OFFSET: usize = 1;

// TESTS
// ================================================================================================

#[test]
fn join_block() {
    let span1 = CodeBlock::new_span(vec![Operation::Push(Felt::ONE)]);
    let span2 = CodeBlock::new_span(vec![Operation::Drop]);
    let program = CodeBlock::new_join([span1, span2]);

    let trace = build_trace(&[], &program);
    let trace_len = trace[0].len();

    // --- test op bits columns -----------------------------------------------

    // opcodes should be: JOIN SPAN PUSH END SPAN DROP END END
    assert!(contains_op(&trace, 0, Operation::Join));
    assert!(contains_op(&trace, 1, Operation::Span));
    assert!(contains_op(&trace, 2, Operation::Push(Felt::ONE)));
    assert!(contains_op(&trace, 3, Operation::End));
    assert!(contains_op(&trace, 4, Operation::Span));
    assert!(contains_op(&trace, 5, Operation::Drop));
    assert!(contains_op(&trace, 6, Operation::End));
    assert!(contains_op(&trace, 7, Operation::End));

    // all remaining opcodes should be HALT
    for i in 8..trace_len - NUM_RAND_ROWS {
        assert!(contains_op(&trace, i, Operation::Halt));
    }
}

// HELPER FUNCTIONS
// ================================================================================================

fn build_trace(stack: &[u64], program: &CodeBlock) -> DecoderTrace {
    let inputs = ProgramInputs::new(stack, &[], vec![]).unwrap();
    let mut process = Process::new(inputs);
    process.execute_code_block(&program).unwrap();

    let trace = ExecutionTrace::test_finalize_trace(process);
    trace[DECODER_TRACE_RANGE]
        .to_vec()
        .try_into()
        .expect("failed to convert vector to array")
}

fn contains_op(trace: &DecoderTrace, row_idx: usize, op: Operation) -> bool {
    op.op_code().unwrap() == read_opcode(trace, row_idx)
}

fn read_opcode(trace: &DecoderTrace, row_idx: usize) -> u8 {
    let mut result = 0;
    for (i, column) in trace
        .iter()
        .skip(OP_BITS_OFFSET)
        .take(NUM_OP_BITS)
        .enumerate()
    {
        let op_bit = column[row_idx].as_int();
        assert!(op_bit <= 1, "invalid op bit");
        result += op_bit << i;
    }
    result as u8
}
