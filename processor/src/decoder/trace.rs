use super::{Felt, Operation, Word, HASHER_WIDTH, MIN_TRACE_LEN, NUM_OP_BITS};
use vm_core::{utils::new_array_vec, FieldElement, StarkField};

// CONSTANTS
// ================================================================================================

// TODO: get from core
const OP_BATCH_SIZE: usize = 8;

const OP_GROUP_IDX: usize = 0;

// DECODER TRACE
// ================================================================================================

pub struct DecoderTrace {
    addr_trace: Vec<Felt>,
    op_bits_trace: [Vec<Felt>; NUM_OP_BITS],
    in_span_trace: Vec<Felt>,
    hasher_trace: [Vec<Felt>; HASHER_WIDTH],
    group_count_trace: Vec<Felt>,
    span_context: SpanContext,
}

impl DecoderTrace {
    pub fn new() -> Self {
        Self {
            addr_trace: Vec::with_capacity(MIN_TRACE_LEN),
            op_bits_trace: new_array_vec(MIN_TRACE_LEN),
            in_span_trace: Vec::with_capacity(MIN_TRACE_LEN),
            hasher_trace: new_array_vec(MIN_TRACE_LEN),
            group_count_trace: Vec::with_capacity(MIN_TRACE_LEN),
            span_context: SpanContext::default(),
        }
    }

    // TRACE MUTATORS
    // --------------------------------------------------------------------------------------------

    pub fn append_row(&mut self, addr: Felt, op: Operation, h1: Word, h2: Word) {
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

        self.in_span_trace.push(Felt::ZERO);
        self.group_count_trace.push(Felt::ZERO);
    }

    /// Append a trace row marking the start of a SPAN block.
    ///
    /// When a SPAN block is starting, we do the following:
    /// - Set the address to the address of the parent block. This is not necessarily equal to the
    ///   address from the previous row because in a SPLIT block, the second child follows the
    ///   first child, rather than the parent.
    /// - Set op_bits to SPAN opcode.
    /// - Set is_span to ZERO. is_span will be set to one in the following row.
    /// - Set hasher state to op groups of the first op batch of the SPAN.
    /// - Set op group count to the total number of op groups in the SPAN.
    pub fn append_span_start(
        &mut self,
        parent_addr: Felt,
        span_addr: Felt,
        first_op_batch: &[Felt; OP_BATCH_SIZE],
        num_op_groups: Felt,
    ) {
        self.addr_trace.push(parent_addr);
        self.append_opcode(Operation::Span);
        self.in_span_trace.push(Felt::ZERO);
        for (i, &op_group) in first_op_batch.iter().enumerate() {
            self.hasher_trace[i].push(op_group);
        }
        self.group_count_trace.push(num_op_groups);

        // set span cursor to the new span and decrement op group count as we immediately start
        // reading the first group of the first op batch in the span
        self.span_context.new_span(span_addr);
    }

    /// Append a trace row for a user operation.
    ///
    /// When we execute a user operation in a SPAN block, we do the following:
    /// - Set the address of the row to the address of the span block.
    /// - Set op_bits to the opcode of the executed operation.
    /// - Set is_span to ONE.
    /// - Subtract the opcode value from the last op group value (which is located in the first
    ///   register of the hasher state) and divide the result by 2^7.
    /// - Set the remaining registers of the hasher state to ZEROs.
    /// - Decrement op group count if this was specified by the previously executed operation.
    pub fn append_user_op(&mut self, op: Operation, num_groups_left: Felt, group_ops_left: Felt) {
        // set span address
        self.addr_trace.push(self.span_context.addr());
        self.append_opcode(op);
        self.in_span_trace.push(Felt::ONE);

        self.group_count_trace.push(num_groups_left);
        self.hasher_trace[OP_GROUP_IDX].push(group_ops_left);

        for column in self.hasher_trace.iter_mut().skip(1) {
            column.push(Felt::ZERO);
        }
    }

    ///
    pub fn append_respan(&mut self, op_batch: &[Felt; OP_BATCH_SIZE]) {
        self.addr_trace.push(self.last_addr());
        self.append_opcode(Operation::Respan);
        self.in_span_trace.push(Felt::ONE);

        for (i, &op_group) in op_batch.iter().enumerate() {
            self.hasher_trace[i].push(op_group);
        }

        self.group_count_trace.push(self.last_group_count());

        self.span_context.respan();
    }

    /// Append a trace row marking the end of a SPAN block.
    ///
    /// When the SPAN block is ending, we do the following things:
    /// - Copy over the block address from the previous row.
    /// - Set op_bits to END opcode.
    /// - Set in_span to ZERO to indicate that the span block is completed.
    /// - Put the hash of the span block into the first 4 registers of the hasher state.
    /// - Put a flag indicating whether the SPAN block was a body of a loop into the 5 register
    ///   of the hasher state.
    /// - Copy over op group count from the previous row. This group count must be ZERO.
    pub fn append_span_end(&mut self, span_hash: Word, is_loop_body: Felt) {
        debug_assert!(is_loop_body.as_int() <= 1, "invalid loop body");

        self.addr_trace.push(self.last_addr());
        self.append_opcode(Operation::End);
        self.in_span_trace.push(Felt::ZERO);

        // put span block hash into the first 4 elements of the hasher state
        for (column, value) in self.hasher_trace.iter_mut().zip(span_hash) {
            column.push(value);
        }

        // set the remaining 4 elements of the hasher state to [is_loop_body, 0, 0, 0]
        let block_flags = [is_loop_body, Felt::ZERO, Felt::ZERO, Felt::ZERO];
        for (column, value) in self.hasher_trace.iter_mut().skip(4).zip(block_flags) {
            column.push(value);
        }

        let last_group_count = self.last_group_count();
        // TODO: debug_assert!(last_group_count == Felt::ZERO, "group count not zero");
        self.group_count_trace.push(last_group_count);
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

        self.in_span_trace.resize(trace_len, Felt::ZERO);
        trace.push(self.in_span_trace);

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

    fn last_addr(&self) -> Felt {
        *self.addr_trace.last().expect("no last addr")
    }

    #[allow(dead_code)]
    fn last_op_group(&self) -> Felt {
        *self.hasher_trace[OP_GROUP_IDX].last().expect("no op group")
    }

    fn last_group_count(&self) -> Felt {
        *self.group_count_trace.last().expect("no group count")
    }

    fn append_opcode(&mut self, op: Operation) {
        let op_code = op.op_code().expect("missing opcode");
        for i in 0..NUM_OP_BITS {
            let bit = Felt::from((op_code >> i) & 1);
            self.op_bits_trace[i].push(bit);
        }
    }
}

// SPAN CONTEXT
// ================================================================================================

struct SpanContext {
    addr: Felt,
}

impl SpanContext {
    pub fn new_span(&mut self, addr: Felt) {
        self.addr = addr;
    }

    pub fn respan(&mut self) {}

    pub fn addr(&self) -> Felt {
        self.addr
    }
}

impl Default for SpanContext {
    fn default() -> Self {
        Self { addr: Felt::ZERO }
    }
}
