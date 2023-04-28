use super::{
    build_trace_from_ops, rand_array, ExecutionTrace, Felt, FieldElement, Operation, Trace, Word,
    AUX_TRACE_RAND_ELEMENTS, CHIPLETS_AUX_TRACE_OFFSET, NUM_RAND_ROWS, ONE, ZERO,
};
use vm_core::chiplets::{
    memory::{MEMORY_READ_LABEL, MEMORY_WRITE, MEMORY_WRITE_LABEL, NUM_ELEMENTS},
    MEMORY_ADDR_COL_IDX, MEMORY_CLK_COL_IDX, MEMORY_CTX_COL_IDX, MEMORY_SELECTORS_COL_IDX,
    MEMORY_V_COL_RANGE,
};

/// Tests the generation of the `b_aux` bus column when only memory lookups are included. It ensures
/// that trace generation is correct when all of the following are true.
///
/// - All possible memory operations are called by the stack.
/// - Some requests from the Stack and responses from Memory occur at the same cycle.
/// - Multiple memory addresses are used.
///
/// Note: Communication with the Hash chiplet is also required, due to the span block decoding, but
/// for this test we set those values explicitly, enforcing only that the same initial and final
/// values are requested & provided.
#[test]
#[allow(clippy::needless_range_loop)]
fn b_aux_trace_mem() {
    let stack = [1, 2, 3, 4, 0];
    let word = [ONE, Felt::new(2), Felt::new(3), Felt::new(4)];
    let operations = vec![
        Operation::MStoreW, // store [1, 2, 3, 4]
        Operation::Drop,    // clear the stack
        Operation::Drop,
        Operation::Drop,
        Operation::Drop,
        Operation::MLoad,     // read the first value of the word
        Operation::MovDn5,    // put address 0 and space for a full word at top of stack
        Operation::MLoadW,    // load word from address 0 to stack
        Operation::Push(ONE), // push a new value onto the stack
        Operation::Push(ONE), // push a new address on to the stack
        Operation::MStore,    // store 1 at address 1
        Operation::Drop,      // ensure the stack overflow table is empty
        Operation::MStream,   // read 2 words starting at address 0
    ];
    let mut trace = build_trace_from_ops(operations, &stack);

    let rand_elements = rand_array::<Felt, AUX_TRACE_RAND_ELEMENTS>();
    let aux_columns = trace.build_aux_segment(&[], &rand_elements).unwrap();
    let b_aux = aux_columns.get_column(CHIPLETS_AUX_TRACE_OFFSET);

    assert_eq!(trace.length(), b_aux.len());
    assert_eq!(ONE, b_aux[0]);

    // At cycle 0 the span hash initialization is requested from the decoder and provided by the
    // hash chiplet, so the trace should still equal one.
    assert_eq!(ONE, b_aux[1]);

    // The first memory request from the stack is sent when the `MStoreW` operation is executed, at
    // cycle 1, so the request is included in the next row. (The trace begins by executing `span`).
    let value = build_expected_memory(&rand_elements, MEMORY_WRITE_LABEL, ZERO, ZERO, ONE, word);
    let mut expected = value.inv();
    assert_eq!(expected, b_aux[2]);

    // Nothing changes after user operations that don't make requests to the Chiplets.
    for row in 3..7 {
        assert_eq!(expected, b_aux[row]);
    }

    // The next memory request from the stack is sent when `MLoad` is executed at cycle 6 and
    // included at row 7
    let value =
        build_expected_memory(&rand_elements, MEMORY_READ_LABEL, ZERO, ZERO, Felt::new(6), word);
    expected *= value.inv();
    assert_eq!(expected, b_aux[7]);

    // At cycle 7 the hasher provides the result of the `SPAN` hash. Since this test is for changes
    // from memory lookups, just set it explicitly and save the multiplied-in value for later.
    assert_ne!(expected, b_aux[8]);
    let span_result = b_aux[8] * b_aux[7].inv();
    expected = b_aux[8];

    // Memory responses will be provided during the memory segment of the Chiplets trace,
    // which starts after the hash for the span block at row 8. There will be 6 rows, corresponding
    // to the 5 memory operations (MStream requires 2 rows).

    // At cycle 8 `MLoadW` is requested by the stack and `MStoreW` is provided by memory
    let value =
        build_expected_memory(&rand_elements, MEMORY_READ_LABEL, ZERO, ZERO, Felt::new(8), word);
    expected *= value.inv();
    expected *= build_expected_memory_from_trace(&trace, &rand_elements, 8);
    assert_eq!(expected, b_aux[9]);

    // At cycle 9, `MLoad` is provided by memory.
    expected *= build_expected_memory_from_trace(&trace, &rand_elements, 9);
    assert_eq!(expected, b_aux[10]);

    // At cycle 10,  `MLoadW` is provided by memory.
    expected *= build_expected_memory_from_trace(&trace, &rand_elements, 10);
    assert_eq!(expected, b_aux[11]);

    // At cycle 11, `MStore` is requested by the stack and the first read of `MStream` is provided
    // by the memory.
    let value = build_expected_memory(
        &rand_elements,
        MEMORY_WRITE_LABEL,
        ZERO,
        ONE,
        Felt::new(11),
        [ONE, ZERO, ZERO, ZERO],
    );
    expected *= value.inv();
    expected *= build_expected_memory_from_trace(&trace, &rand_elements, 11);
    assert_eq!(expected, b_aux[12]);

    // At cycle 12, `MStore` is provided by the memory
    expected *= build_expected_memory_from_trace(&trace, &rand_elements, 12);
    assert_eq!(expected, b_aux[13]);

    // At cycle 13, `MStream` is requested by the stack, and the second read of `MStream` is
    // provided by the memory.
    let value1 =
        build_expected_memory(&rand_elements, MEMORY_READ_LABEL, ZERO, ZERO, Felt::new(13), word);
    let value2 = build_expected_memory(
        &rand_elements,
        MEMORY_READ_LABEL,
        ZERO,
        ONE,
        Felt::new(13),
        [ONE, ZERO, ZERO, ZERO],
    );
    expected *= (value1 * value2).inv();
    expected *= build_expected_memory_from_trace(&trace, &rand_elements, 13);
    assert_eq!(expected, b_aux[14]);

    // At cycle 14 the decoder requests the span hash. We set this as the inverse of the previously
    // identified `span_result`, since this test is for consistency of the memory lookups.
    assert_ne!(expected, b_aux[15]);
    expected *= span_result.inv();
    assert_eq!(expected, b_aux[15]);

    // The value in b_aux should be ONE now and for the rest of the trace.
    for row in 15..trace.length() - NUM_RAND_ROWS {
        assert_eq!(ONE, b_aux[row]);
    }
}

// TEST HELPERS
// ================================================================================================

fn build_expected_memory(
    alphas: &[Felt],
    op_label: u8,
    ctx: Felt,
    addr: Felt,
    clk: Felt,
    word: Word,
) -> Felt {
    let mut word_value = ZERO;
    for i in 0..NUM_ELEMENTS {
        word_value += alphas[i + 5] * word[i];
    }

    alphas[0]
        + alphas[1] * Felt::from(op_label)
        + alphas[2] * ctx
        + alphas[3] * addr
        + alphas[4] * clk
        + word_value
}

fn build_expected_memory_from_trace(trace: &ExecutionTrace, alphas: &[Felt], row: usize) -> Felt {
    // get the memory access operation
    let s0 = trace.main_trace.get_column(MEMORY_SELECTORS_COL_IDX)[row];
    let s1 = trace.main_trace.get_column(MEMORY_SELECTORS_COL_IDX + 1)[row];
    let op_label = if s0 == MEMORY_WRITE[0] {
        debug_assert!(s1 == ZERO);
        MEMORY_WRITE_LABEL
    } else {
        MEMORY_READ_LABEL
    };

    // get the memory access data
    let ctx = trace.main_trace.get_column(MEMORY_CTX_COL_IDX)[row];
    let addr = trace.main_trace.get_column(MEMORY_ADDR_COL_IDX)[row];
    let clk = trace.main_trace.get_column(MEMORY_CLK_COL_IDX)[row];

    // get the memory value
    let mut word = [ZERO; NUM_ELEMENTS];
    for (i, element) in word.iter_mut().enumerate() {
        *element = trace.main_trace.get_column(MEMORY_V_COL_RANGE.start + i)[row];
    }

    build_expected_memory(alphas, op_label, ctx, addr, clk, word)
}
