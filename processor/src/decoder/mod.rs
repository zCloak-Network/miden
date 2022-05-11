use vm_core::FieldElement;
use super::{ExecutionError, Felt, Join, Loop, OpBatch, Operation, Process, Span, Split};

// DECODER PROCESS EXTENSION
// ================================================================================================

impl Process {
    // JOIN BLOCK
    // --------------------------------------------------------------------------------------------

    pub(super) fn start_join_block(&mut self, block: &Join) -> Result<(), ExecutionError> {
        self.execute_op(Operation::Noop)?;

        // TODO: get address from hasher
        let addr = Felt::ZERO;
        self.decoder.start_join(block, addr);

        Ok(())
    }

    pub(super) fn end_join_block(&mut self, block: &Join) -> Result<(), ExecutionError> {
        self.execute_op(Operation::Noop)?;

        self.decoder.end_join(block);

        Ok(())
    }

    // SPLIT BLOCK
    // --------------------------------------------------------------------------------------------

    pub(super) fn start_split_block(&mut self, block: &Split) -> Result<(), ExecutionError> {
        let condition = self.stack.peek();
        self.execute_op(Operation::Drop)?;

        // TODO: get address from hasher
        let addr = Felt::ZERO;
        self.decoder.start_split(block, addr, condition);

        Ok(())
    }

    pub(super) fn end_split_block(&mut self, block: &Split) -> Result<(), ExecutionError> {
        self.execute_op(Operation::Noop)?;

        self.decoder.end_split(block);

        Ok(())
    }

    // SPAN BLOCK
    // --------------------------------------------------------------------------------------------

    pub(super) fn start_span_block(&mut self, block: &Span) -> Result<(), ExecutionError> {
        self.execute_op(Operation::Noop)?;

        // TODO: get address from hasher
        let addr = Felt::ZERO;
        self.decoder.start_span(block, addr);

        Ok(())
    }

    pub(super) fn end_span_block(&mut self, block: &Span) -> Result<(), ExecutionError> {
        self.execute_op(Operation::Noop)?;

        self.decoder.end_span(block);

        Ok(())
    }
}

// DECODER
// ================================================================================================
/// TODO: add docs
pub struct Decoder {
    addr_trace: Vec<Felt>,
    op_bits_trace: [Vec<Felt>; NUM_OP_BITS],
    is_span_trace: Vec<Felt>,
    hasher_trace: [Vec<Felt>; HASHER_WIDTH],
    group_count_trace: Vec<Felt>,
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            addr_trace: Vec::with_capacity(MIN_TRACE_LEN),
            op_bits_trace: new_array_vec(MIN_TRACE_LEN),
            is_span_trace: Vec::with_capacity(MIN_TRACE_LEN),
            hasher_trace: new_array_vec(MIN_TRACE_LEN),
            group_count_trace: Vec::with_capacity(MIN_TRACE_LEN),
        }
    }

    // JOIN BLOCK
    // --------------------------------------------------------------------------------------------

    pub fn start_join(&mut self, _block: &Join, _addr: Felt) {}

    pub fn end_join(&mut self, _block: &Join) {
        self.append_opcode(Operation::End);
    }

    // SPLIT BLOCK
    // --------------------------------------------------------------------------------------------

    pub fn start_split(&mut self, _block: &Split, _addr: Felt, _condition: Felt) {}

    pub fn end_split(&mut self, _block: &Split) {
        self.append_opcode(Operation::End);
    }

    // LOOP BLOCK
    // --------------------------------------------------------------------------------------------

    pub fn start_loop(&mut self, _block: &Loop, _condition: Felt) {
        self.append_opcode(Operation::Loop);
    }

    pub fn repeat(&mut self, _block: &Loop) {
        self.append_opcode(Operation::Repeat);
    }

    pub fn end_loop(&mut self, _block: &Loop) {
        self.append_opcode(Operation::End);
    }

    // SPAN BLOCK
    // --------------------------------------------------------------------------------------------
    pub fn start_span(&mut self, _block: &Span, _addr: Felt) {}

    pub fn respan(&mut self, _op_batch: &OpBatch) {
        self.append_opcode(Operation::Respan);
    }

    pub fn execute_user_op(&mut self, op: Operation) {
        if !op.is_decorator() {
            self.append_opcode(op);
        }
    }

    pub fn end_span(&mut self, _block: &Span) {
        self.append_opcode(Operation::End);
    }

    // TRACE GENERATIONS
    // --------------------------------------------------------------------------------------------

    /// TODO: add docs
    pub fn into_trace(mut self, trace_len: usize, _num_rand_rows: usize) -> DecoderTrace {
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

        trace.try_into().expect("failed to convert vector to array")
    }

    // HELPER FUNCTIONS
    // --------------------------------------------------------------------------------------------

    fn append_opcode(&mut self, op: Operation) {
        let op_code = op.op_code().expect("missing opcode");
        for i in 0..7 {
            let bit = Felt::from((op_code >> i) & 1);
            self.op_bits_trace[i].push(bit);
        }
    }
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}
