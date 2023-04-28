use super::{Felt, FieldElement, Vec, MAX_TOP_IDX, ONE, STACK_TRACE_WIDTH, ZERO};
use crate::utils::get_trace_len;
use vm_core::{
    stack::{H0_COL_IDX, NUM_STACK_HELPER_COLS, STACK_TOP_SIZE},
    utils::math::batch_inversion,
};

// STACK TRACE
// ================================================================================================

/// Execution trace of the stack component.
///
/// The trace consists of 19 columns grouped logically as follows:
/// - 16 stack columns holding the top of the stack.
/// - 3 columns for bookkeeping and helper values that manage left and right shifts.
pub struct StackTrace {
    stack: [Vec<Felt>; STACK_TOP_SIZE],
    helpers: [Vec<Felt>; NUM_STACK_HELPER_COLS],
}

impl StackTrace {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a [StackTrace] instantiated with the provided input values.
    ///
    /// When fewer than `STACK_TOP_SIZE` inputs are provided, the rest of the stack top elements
    /// are set to ZERO. The initial stack depth and initial overflow address are used to
    /// initialize the bookkeeping columns so they are consistent with the initial state of the
    /// overflow table.
    pub fn new(
        init_values: &[Felt],
        init_trace_capacity: usize,
        init_depth: usize,
        init_overflow_addr: Felt,
    ) -> Self {
        StackTrace {
            stack: init_stack_columns(init_trace_capacity, init_values),
            helpers: init_helper_columns(init_trace_capacity, init_depth, init_overflow_addr),
        }
    }

    // STACK ACCESSORS AND MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Returns a copy of the item at the top of the stack at the specified clock cycle.
    #[inline(always)]
    pub fn peek_at(&self, clk: u32) -> Felt {
        self.stack[0][clk as usize]
    }

    /// Returns the value located at the specified position on the stack at the specified clock
    /// cycle.
    #[inline(always)]
    pub fn get_stack_value_at(&self, clk: u32, pos: usize) -> Felt {
        self.stack[pos][clk as usize]
    }

    /// Sets the value at the specified position on the stack at the specified cycle.
    #[inline(always)]
    pub fn set_stack_value_at(&mut self, clk: u32, pos: usize, value: Felt) {
        self.stack[pos][clk as usize] = value;
    }

    /// Copies the stack values starting at the specified position at the specified clock cycle to
    /// the same position at the next clock cycle.
    ///
    /// Also, sets values in the stack helper columns for the next clock cycle to the provided
    /// stack depth and overflow address.
    pub fn copy_stack_state_at(
        &mut self,
        clk: u32,
        start_pos: usize,
        stack_depth: Felt,
        next_overflow_addr: Felt,
    ) {
        let clk = clk as usize;

        // copy over stack top columns
        for i in start_pos..STACK_TOP_SIZE {
            self.stack[i][clk + 1] = self.stack[i][clk];
        }

        // update stack helper columns
        self.set_helpers_at(clk, stack_depth, next_overflow_addr);
    }

    /// Copies the stack values starting at the specified position at the specified clock cycle to
    /// position - 1 at the next clock cycle.
    ///
    /// The final stack item column is filled with the provided value in `last_value`.
    ///
    /// If next_overflow_addr is provided, this function assumes that the stack depth has been
    /// decreased by one and a row has been removed from the overflow table. Thus, it makes the
    /// following changes to the helper columns:
    /// - Decrement the stack depth (b0) by one.
    /// - Sets b1 to the address of the top row in the overflow table to the specified
    ///   `next_overflow_addr`.
    /// - Set h0 to (depth - 16). Inverses of these values will be computed in into_array() method
    ///   after the entire trace is constructed.
    pub fn stack_shift_left_at(
        &mut self,
        clk: u32,
        start_pos: usize,
        last_value: Felt,
        next_overflow_addr: Option<Felt>,
    ) {
        let clk = clk as usize;

        // update stack top columns
        for i in start_pos..=MAX_TOP_IDX {
            self.stack[i - 1][clk + 1] = self.stack[i][clk];
        }
        self.stack[MAX_TOP_IDX][clk + 1] = last_value;

        // update stack helper columns
        if let Some(next_overflow_addr) = next_overflow_addr {
            let next_depth = self.helpers[0][clk] - ONE;
            self.set_helpers_at(clk, next_depth, next_overflow_addr);
        } else {
            // if next_overflow_addr was not provide, just copy over the values from the last row
            let next_depth = self.helpers[0][clk];
            let next_overflow_addr = self.helpers[1][clk];
            self.set_helpers_at(clk, next_depth, next_overflow_addr);
        }
    }

    /// Copies stack values starting at the specified position at the specified clock cycle to
    /// position + 1 at the next clock cycle.
    ///
    /// This function assumes that the stack depth has been increased by one and a new row has been
    /// added to the overflow table. It also makes the following changes to the helper columns:
    /// - Increments the stack depth (b0) by one.
    /// - Sets b1 to the address of the new top row in overflow table, which is the current clock
    ///   cycle.
    /// - Set h0 to (depth - 16). Inverses of these values will be computed in into_array() method
    ///   after the entire trace is constructed.
    pub fn stack_shift_right_at(&mut self, clk: u32, start_pos: usize) {
        let clk = clk as usize;

        // update stack top columns
        for i in start_pos..MAX_TOP_IDX {
            self.stack[i + 1][clk + 1] = self.stack[i][clk];
        }

        // update stack helper columns
        let next_depth = self.helpers[0][clk] + ONE;
        self.set_helpers_at(clk, next_depth, Felt::from(clk as u32));
    }

    // UTILITY METHODS
    // --------------------------------------------------------------------------------------------

    /// Makes sure there is enough memory allocated for the trace to accommodate a new row.
    ///
    /// Trace length is doubled every time it needs to be increased.
    pub fn ensure_trace_capacity(&mut self, clk: u32) {
        let current_capacity = get_trace_len(&self.stack);
        // current_capacity as trace_length can not be bigger than clk, so it is safe to cast to u32
        if clk + 1 >= current_capacity as u32 {
            let new_length = current_capacity * 2;
            for column in self.stack.iter_mut().chain(self.helpers.iter_mut()) {
                column.resize(new_length, ZERO);
            }
        }
    }

    /// Appends stack top state (16 items) at the specified clock cycle into the provided vector.
    pub fn append_state_into(&self, result: &mut Vec<Felt>, clk: u32) {
        for column in self.stack.iter() {
            result.push(column[clk as usize]);
        }
    }

    /// Combines all columns of the trace (stack + helpers) into a single array of vectors.
    pub fn into_array(self) -> [Vec<Felt>; STACK_TRACE_WIDTH] {
        let mut trace = Vec::with_capacity(STACK_TRACE_WIDTH);
        self.stack.into_iter().for_each(|col| trace.push(col));
        self.helpers.into_iter().for_each(|col| trace.push(col));

        // compute inverses in the h0 helper column using batch inversion; any ZERO in the vector
        // will remain unchanged
        trace[H0_COL_IDX] = batch_inversion(&trace[H0_COL_IDX]);

        trace.try_into().expect("Failed to convert vector to an array")
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------

    /// Sets values of stack helper columns for the next clock cycle. Note that h0 column value is
    /// set to (stack_depth - 16) rather than to 1 / (stack_depth - 16). Inverses of these values
    /// will be computed in into_array() method (using batch inversion) after the entire trace is
    /// constructed.
    fn set_helpers_at(&mut self, clk: usize, stack_depth: Felt, next_overflow_addr: Felt) {
        self.helpers[0][clk + 1] = stack_depth;
        self.helpers[1][clk + 1] = next_overflow_addr;
        self.helpers[2][clk + 1] = stack_depth - Felt::from(STACK_TOP_SIZE as u32);
    }

    // TEST HELPERS
    // --------------------------------------------------------------------------------------------

    /// Returns the stack trace state at the specified clock cycle.
    #[cfg(any(test, feature = "internals"))]
    pub fn get_stack_state_at(&self, clk: u32) -> [Felt; STACK_TOP_SIZE] {
        let mut result = [ZERO; STACK_TOP_SIZE];
        for (result, column) in result.iter_mut().zip(self.stack.iter()) {
            *result = column[clk as usize];
        }
        result
    }

    /// Returns the trace state of the stack helper columns at the specified clock cycle.
    #[cfg(test)]
    pub fn get_helpers_state_at(&self, clk: u32) -> [Felt; NUM_STACK_HELPER_COLS] {
        let mut result = [ZERO; NUM_STACK_HELPER_COLS];
        for (result, column) in result.iter_mut().zip(self.helpers.iter()) {
            *result = column[clk as usize];
        }
        result
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Initializes the 16 stack top columns.
fn init_stack_columns(
    init_trace_capacity: usize,
    init_values: &[Felt],
) -> [Vec<Felt>; STACK_TOP_SIZE] {
    let mut stack: Vec<Vec<Felt>> = Vec::with_capacity(STACK_TOP_SIZE);
    for i in 0..STACK_TOP_SIZE {
        let mut column = Felt::zeroed_vector(init_trace_capacity);
        if i < init_values.len() {
            column[0] = init_values[i];
        }
        stack.push(column)
    }

    stack.try_into().expect("Failed to convert vector to an array")
}

/// Initializes the bookkeeping & helper columns.
fn init_helper_columns(
    init_trace_capacity: usize,
    init_depth: usize,
    init_overflow_addr: Felt,
) -> [Vec<Felt>; NUM_STACK_HELPER_COLS] {
    // initialize b0 to the initial stack depth.
    let mut b0 = Felt::zeroed_vector(init_trace_capacity);
    b0[0] = Felt::new(init_depth as u64);

    // initialize b1 to the address of the last row in the stack overflow table.
    let mut b1 = Felt::zeroed_vector(init_trace_capacity);
    b1[0] = init_overflow_addr;

    // if the overflow table is not empty, set h0 to (init_depth - 16)
    let mut h0 = Felt::zeroed_vector(init_trace_capacity);
    h0[0] = Felt::from((init_depth - STACK_TOP_SIZE) as u64);

    [b0, b1, h0]
}
