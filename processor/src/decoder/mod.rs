use super::{
    ExecutionError, Felt, Join, Loop, OpBatch, Operation, Process, Span, Split, MIN_TRACE_LEN,
};
use vm_core::{FieldElement, Word};

mod trace;
use trace::DecoderTrace;

// DECODER PROCESS EXTENSION
// ================================================================================================

impl Process {
    // JOIN BLOCK
    // --------------------------------------------------------------------------------------------

    pub(super) fn start_join_block(&mut self, block: &Join) -> Result<(), ExecutionError> {
        self.execute_op(Operation::Noop)?;

        let hasher_state = [Felt::ZERO; 12];
        let (addr, _result) = self.hasher.hash(hasher_state);
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

        let hasher_state = [Felt::ZERO; 12];
        let (addr, _result) = self.hasher.hash(hasher_state);
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

        let hasher_state = [Felt::ZERO; 12];
        let (addr, _result) = self.hasher.hash(hasher_state);
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
    block_stack: BlockStack,
    trace: DecoderTrace,
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            block_stack: BlockStack::new(),
            trace: DecoderTrace::new(),
        }
    }

    // JOIN BLOCK
    // --------------------------------------------------------------------------------------------

    pub fn start_join(&mut self, block: &Join, addr: Felt) {
        let parent_addr = self.block_stack.push(addr);
        let left_child_hash: Word = block.first().hash().into();
        let right_child_hash: Word = block.second().hash().into();
        self.trace
            .append_join_row(parent_addr, left_child_hash, right_child_hash);
    }

    pub fn end_join(&mut self, block: &Join) {
        let block_info = self.block_stack.pop();
        let block_hash: Word = block.hash().into();
        self.trace.append_end_row(block_info.addr, block_hash);
    }

    // SPLIT BLOCK
    // --------------------------------------------------------------------------------------------

    pub fn start_split(&mut self, block: &Split, addr: Felt, _condition: Felt) {
        let parent_addr = self.block_stack.push(addr);
        let left_child_hash: Word = block.on_true().hash().into();
        let right_child_hash: Word = block.on_false().hash().into();
        self.trace
            .append_split_row(parent_addr, left_child_hash, right_child_hash);
    }

    pub fn end_split(&mut self, block: &Split) {
        let block_info = self.block_stack.pop();
        let block_hash: Word = block.hash().into();
        self.trace.append_end_row(block_info.addr, block_hash);
    }

    // LOOP BLOCK
    // --------------------------------------------------------------------------------------------

    pub fn start_loop(&mut self, _block: &Loop, _condition: Felt) {
        // TODO: implement
    }

    pub fn repeat(&mut self, _block: &Loop) {
        // TODO: implement
    }

    pub fn end_loop(&mut self, _block: &Loop) {
        // TODO: implement
    }

    // SPAN BLOCK
    // --------------------------------------------------------------------------------------------
    pub fn start_span(&mut self, _block: &Span, addr: Felt) {
        let parent_addr = self.block_stack.push(addr);
        self.trace.append_span_row(parent_addr);
    }

    pub fn respan(&mut self, _op_batch: &OpBatch) {
        // TODO: implement
    }

    pub fn execute_user_op(&mut self, op: Operation) {
        if !op.is_decorator() {
            self.trace.append_op_row(self.block_stack.peek_addr(), op);
        }
    }

    pub fn end_span(&mut self, block: &Span) {
        let block_info = self.block_stack.pop();
        let block_hash: Word = block.hash().into();
        self.trace.append_end_row(block_info.addr, block_hash);
    }

    // TRACE GENERATIONS
    // --------------------------------------------------------------------------------------------

    /// TODO: add docs
    pub fn into_trace(self, trace_len: usize, num_rand_rows: usize) -> super::DecoderTrace {
        self.trace
            .into_vec(trace_len, num_rand_rows)
            .try_into()
            .expect("failed to convert vector to array")
    }
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

// BLOCK INFO
// ================================================================================================

pub struct BlockStack {
    blocks: Vec<BlockInfo>,
}

impl BlockStack {
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    pub fn push(&mut self, addr: Felt) -> Felt {
        let parent_addr = if self.blocks.is_empty() {
            Felt::ZERO
        } else {
            self.blocks[self.blocks.len() - 1].addr
        };
        self.blocks.push(BlockInfo { addr, parent_addr });

        parent_addr
    }

    pub fn pop(&mut self) -> BlockInfo {
        self.blocks.pop().expect("block stack is empty")
    }

    pub fn peek_addr(&self) -> Felt {
        self.blocks.last().expect("block stack is empty").addr
    }
}

#[allow(dead_code)]
pub struct BlockInfo {
    addr: Felt,
    parent_addr: Felt,
}
