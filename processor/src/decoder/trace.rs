use super::{Felt, Operation, Word, HASHER_WIDTH, MIN_TRACE_LEN, NUM_OP_BITS};
use vm_core::{utils::new_array_vec, FieldElement};

// DECODER TRACE
// ================================================================================================

pub struct DecoderTrace {
    addr_trace: Vec<Felt>,
    op_bits_trace: [Vec<Felt>; NUM_OP_BITS],
    is_span_trace: Vec<Felt>,
    hasher_trace: [Vec<Felt>; HASHER_WIDTH],
    group_count_trace: Vec<Felt>,
}

impl DecoderTrace {
    pub fn new() -> Self {
        Self {
            addr_trace: Vec::with_capacity(MIN_TRACE_LEN),
            op_bits_trace: new_array_vec(MIN_TRACE_LEN),
            is_span_trace: Vec::with_capacity(MIN_TRACE_LEN),
            hasher_trace: new_array_vec(MIN_TRACE_LEN),
            group_count_trace: Vec::with_capacity(MIN_TRACE_LEN),
        }
    }

    // TRACE MUTATORS
    // --------------------------------------------------------------------------------------------

    pub fn append_join_row(&mut self, addr: Felt, left_child_hash: Word, right_child_hash: Word) {
        self.append_row(addr, Operation::Join, left_child_hash, right_child_hash);
    }

    pub fn append_split_row(&mut self, addr: Felt, left_child_hash: Word, right_child_hash: Word) {
        self.append_row(addr, Operation::Split, left_child_hash, right_child_hash);
    }

    pub fn append_span_row(&mut self, addr: Felt) {
        self.append_row(addr, Operation::Span, [Felt::ZERO; 4], [Felt::ZERO; 4]);
    }

    pub fn append_end_row(&mut self, addr: Felt, block_hash: Word) {
        self.append_row(addr, Operation::End, block_hash, [Felt::ZERO; 4]);
    }

    pub fn append_op_row(&mut self, span_addr: Felt, op: Operation) {
        self.append_row(span_addr, op, [Felt::ZERO; 4], [Felt::ZERO; 4]);
    }

    // TRACE GENERATION
    // --------------------------------------------------------------------------------------------

    /// TODO: add docs
    pub fn into_vec(mut self, trace_len: usize, _num_rand_rows: usize) -> Vec<Vec<Felt>> {
        let mut trace = Vec::new();

        self.addr_trace.resize(trace_len, Felt::ZERO);
        trace.push(self.addr_trace);

        // insert HALT opcode into unfilled rows of ob_bits columns
        let halt_opcode = Operation::Halt.op_code().expect("missing opcode");
        for (i, mut column) in self.op_bits_trace.into_iter().enumerate() {
            let value = Felt::from((halt_opcode >> i) & 1);
            column.resize(trace_len, value);
            trace.push(column);
        }

        self.is_span_trace.resize(trace_len, Felt::ZERO);
        trace.push(self.is_span_trace);

        for mut column in self.hasher_trace {
            column.resize(trace_len, Felt::ZERO);
            trace.push(column);
        }

        self.group_count_trace.resize(trace_len, Felt::ZERO);
        trace.push(self.group_count_trace);

        trace
    }

    // HELPER FUNCTIONS
    // --------------------------------------------------------------------------------------------

    fn append_row(&mut self, addr: Felt, op: Operation, h1: Word, h2: Word) {
        self.addr_trace.push(addr);

        let op_code = op.op_code().expect("missing opcode");
        for i in 0..NUM_OP_BITS {
            let bit = Felt::from((op_code >> i) & 1);
            self.op_bits_trace[i].push(bit);
        }

        for (i, &element) in h1.iter().enumerate() {
            self.hasher_trace[i].push(element);
        }

        for (i, &element) in h2.iter().enumerate() {
            self.hasher_trace[i + 4].push(element);
        }
    }
}
