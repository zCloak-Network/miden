use super::{super::DecoderTrace, Felt, Operation, NUM_OP_BITS};
use crate::{ExecutionTrace, Process, ProgramInputs};
use core::ops::Range;
use vm_core::{program::blocks::CodeBlock, utils::range, StarkField, DECODER_TRACE_RANGE};

// CONSTANTS
// ================================================================================================

/// TODO: move to core?
const OP_BITS_OFFSET: usize = 1;
const OP_BITS_RANGE: Range<usize> = range(OP_BITS_OFFSET, NUM_OP_BITS);

// TESTS
// ================================================================================================

#[test]
fn join_block() {
    let value1 = Felt::new(3);
    let value2 = Felt::new(5);
    let span1 = CodeBlock::new_span(vec![Operation::Push(value1), Operation::Mul]);
    let span2 = CodeBlock::new_span(vec![Operation::Push(value2), Operation::Add]);
    let program = CodeBlock::new_join([span1, span2]);

    let trace = build_trace(&[], &program);
    let _trace_len = trace[0].len();

    // --- test op bits columns -----------------------------------------------

    // opcodes should be: JOIN SPAN PUSH END SPAN DROP END END
    assert!(contains_op(&trace, 0, Operation::Join));
    assert!(contains_op(&trace, 1, Operation::Span));
    assert!(contains_op(&trace, 2, Operation::Push(value1)));
    assert!(contains_op(&trace, 3, Operation::Mul));
    assert!(contains_op(&trace, 4, Operation::End));
    assert!(contains_op(&trace, 5, Operation::Span));
    assert!(contains_op(&trace, 6, Operation::Push(value2)));
    assert!(contains_op(&trace, 7, Operation::Add));
    assert!(contains_op(&trace, 8, Operation::End));
    assert!(contains_op(&trace, 9, Operation::End));
}

#[test]
fn span_block() {
    let program = CodeBlock::new_span(vec![
        Operation::Push(Felt::new(1)),
        Operation::Push(Felt::new(2)),
        Operation::Push(Felt::new(3)),
        Operation::Push(Felt::new(4)),
        Operation::Mul,
        Operation::Add,
        Operation::Drop,
        Operation::Push(Felt::new(5)),
        Operation::Mul,
        Operation::Add,
        Operation::Push(Felt::new(6)),
        Operation::Inv,
    ]);
    //let span2 = CodeBlock::new_span(vec![Operation::Add]);
    //let program = CodeBlock::new_join([span1, span2]);

    let trace = build_trace(&[], &program);
    assert!(contains_op(&trace, 0, Operation::Span));
    //for i in 0..20 {
    //    print_row(&trace, i);
    //}
    //assert!(false, "all good!");
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

#[allow(dead_code)]
fn print_row(trace: &DecoderTrace, idx: usize) {
    let mut row = Vec::new();
    for column in trace.iter() {
        row.push(column[idx].as_int());
    }
    println!(
        "{}\t{}\t{:?} {} {: <16x?} {: <16x?} {}",
        idx,
        row[0],
        &row[OP_BITS_RANGE],
        row[8],
        &row[9..13],
        &row[13..17],
        row[17]
    );
}
