use super::{Felt, FieldElement, StarkField, SysTrace, Vec, Word, ONE, ZERO};

// CONSTANTS
// ================================================================================================

// We assign the following special meanings to memory segments in each context:
// - First 2^30 addresses (0 to 2^30 - 1) are reserved for global memory.
// - The next 2^30 addresses (2^30 to 2^31 - 1) are reserved for procedure locals.
// - The next 2^30 addresses (2^31 to 3 * 2^30 - 1) are reserved for procedure locals of SYSCALLs.
// - All remaining addresses do not have any special meaning.
//
// Note that the above assignment is purely conventional: a program can read from and write to any
// address in a given context, regardless of which memory segment it belongs to.

/// Memory addresses for procedure locals start at 2^30.
pub const FMP_MIN: u64 = 2_u64.pow(30);
/// Memory address for procedure locals within a SYSCALL starts at 2^31.
pub const SYSCALL_FMP_MIN: u64 = 2_u64.pow(31);
/// Value of FMP register should not exceed 3 * 2^30 - 1.
pub const FMP_MAX: u64 = 3 * 2_u64.pow(30) - 1;

// SYSTEM INFO
// ================================================================================================

/// System info container for the VM.
///
/// This keeps track of the following system variables:
/// - clock cycle (clk), which starts at 0 and is incremented with every step.
/// - execution context (ctx), which starts at 0 (root context), and changes when CALL or SYSCALL
///   operations are executed by the VM (or when we return from a CALL or SYSCALL).
/// - free memory pointer (fmp), which is initially set to 2^30.
/// - in_syscall flag which indicates whether the execution is currently in a SYSCALL block.
/// - hash of the function which initiated the current execution context. if the context was
///   initiated from the root context, this will be set to ZEROs.
pub struct System {
    clk: u32,
    ctx: u32,
    fmp: Felt,
    in_syscall: bool,
    fn_hash: Word,
    ctx_trace: Vec<Felt>,
    clk_trace: Vec<Felt>,
    fmp_trace: Vec<Felt>,
    in_syscall_trace: Vec<Felt>,
    fn_hash_trace: [Vec<Felt>; 4],
}

impl System {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new [System] struct with execution traces instantiated with the specified length.
    ///
    /// Initializes the free memory pointer `fmp` used for local memory offsets to 2^30.
    pub fn new(init_trace_capacity: usize) -> Self {
        // set the first value of the fmp trace to 2^30.
        let fmp = Felt::from(FMP_MIN);
        let mut fmp_trace = Felt::zeroed_vector(init_trace_capacity);
        fmp_trace[0] = fmp;

        Self {
            clk: 0,
            ctx: 0,
            fmp,
            in_syscall: false,
            fn_hash: [ZERO; 4],
            clk_trace: Felt::zeroed_vector(init_trace_capacity),
            ctx_trace: Felt::zeroed_vector(init_trace_capacity),
            fmp_trace,
            in_syscall_trace: Felt::zeroed_vector(init_trace_capacity),
            fn_hash_trace: [
                Felt::zeroed_vector(init_trace_capacity),
                Felt::zeroed_vector(init_trace_capacity),
                Felt::zeroed_vector(init_trace_capacity),
                Felt::zeroed_vector(init_trace_capacity),
            ],
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the current clock cycle of a process.
    #[inline(always)]
    pub fn clk(&self) -> u32 {
        self.clk
    }

    /// Returns the current execution context ID.
    #[inline(always)]
    pub fn ctx(&self) -> u32 {
        self.ctx
    }

    /// Returns the current value of the free memory pointer for a process.
    #[inline(always)]
    pub fn fmp(&self) -> Felt {
        self.fmp
    }

    /// Returns true if the VM is currently executing a SYSCALL block.
    pub fn in_syscall(&self) -> bool {
        self.in_syscall
    }

    /// Returns hash of the function which initiated the current execution context.
    #[inline(always)]
    pub fn fn_hash(&self) -> Word {
        self.fn_hash
    }

    /// Returns execution trace length for the systems columns of the process.
    ///
    /// Trace length of the system columns is equal to the number of cycles executed by the VM.
    #[inline(always)]
    pub fn trace_len(&self) -> usize {
        self.clk as usize
    }

    /// Returns execution context ID at the specified clock cycle.
    #[inline(always)]
    pub fn get_ctx_at(&self, clk: u32) -> u32 {
        self.ctx_trace[clk as usize].as_int() as u32
    }

    /// Returns free memory pointer at the specified clock cycle.
    #[inline(always)]
    pub fn get_fmp_at(&self, clk: u32) -> Felt {
        self.fmp_trace[clk as usize]
    }

    // STATE MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Increments the clock cycle.
    pub fn advance_clock(&mut self) {
        self.clk += 1;

        let clk = self.clk as usize;

        self.clk_trace[clk] = Felt::from(self.clk);
        self.fmp_trace[clk] = self.fmp;
        self.ctx_trace[clk] = Felt::from(self.ctx);
        self.in_syscall_trace[clk] = if self.in_syscall { ONE } else { ZERO };

        self.fn_hash_trace[0][clk] = self.fn_hash[0];
        self.fn_hash_trace[1][clk] = self.fn_hash[1];
        self.fn_hash_trace[2][clk] = self.fn_hash[2];
        self.fn_hash_trace[3][clk] = self.fn_hash[3];
    }

    /// Sets the value of free memory pointer for the next clock cycle.
    pub fn set_fmp(&mut self, fmp: Felt) {
        // we set only the current value of fmp here, the trace will be updated with this value
        // when the clock cycle advances.
        self.fmp = fmp;
    }

    /// Updates system registers to mark a new function call.
    ///
    /// Internally, this performs the following updates:
    /// - Set the execution context to the current clock cycle + 1. This ensures that the context
    ///   is globally unique as is never set to 0.
    /// - Sets the free memory pointer to its initial value (FMP_MIN).
    /// - Sets the hash of the function which initiated the current context to the provided value.
    ///
    /// A CALL cannot be started when the VM is executing a SYSCALL.
    pub fn start_call(&mut self, fn_hash: Word) {
        debug_assert!(!self.in_syscall, "call in syscall");
        self.ctx = self.clk + 1;
        self.fmp = Felt::from(FMP_MIN);
        self.fn_hash = fn_hash;
    }

    /// Updates system registers to mark a new syscall.
    ///
    /// Internally, this performs the following updates:
    /// - Set the execution context to 0 (the root context).
    /// - Sets the free memory pointer to the initial value of syscalls (SYSCALL_FMP_MIN). This
    ///   ensures that procedure locals within a syscall do not conflict with procedure locals
    ///   of the original root context.
    /// - Sets the in_syscall flag to true.
    ///
    /// A SYSCALL cannot be started when the VM is executing a SYSCALL.
    ///
    /// Note that this does not change the hash of the function which initiated the context:
    /// for SYSCALLs this remains set to the hash of the last invoked function.
    pub fn start_syscall(&mut self) {
        debug_assert!(!self.in_syscall, "already in syscall");
        self.ctx = 0;
        self.fmp = Felt::from(SYSCALL_FMP_MIN);
        self.in_syscall = true;
    }

    /// Updates system registers to the provided values. These updates are made at the end of a
    /// CALL or a SYSCALL blocks.
    ///
    /// Note that we set in_syscall flag to true regardless of whether we return from a CALL or a
    /// SYSCALL.
    pub fn restore_context(&mut self, ctx: u32, fmp: Felt, fn_hash: Word) {
        self.ctx = ctx;
        self.fmp = fmp;
        self.in_syscall = false;
        self.fn_hash = fn_hash;
    }

    // TRACE GENERATIONS
    // --------------------------------------------------------------------------------------------

    /// Returns an execution trace of this system info container.
    ///
    /// If the trace is smaller than the specified `trace_len`, the columns of the trace are
    /// extended to match the specified length as follows:
    /// - the remainder of the `clk` column is filled in with increasing values of `clk`.
    /// - the remainder of the `ctx` column is filled in with ZERO, which should be the last value
    ///   in the column.
    /// - the remainder of the `fmp` column is filled in with the last value in the column.
    /// - the remainder of the `in_syscall` column is filled with ZERO, which should be the last
    ///   value in this colum.
    /// - the remainder of the `fn_hash` columns are filled with ZERO, which should be the last
    ///   values in these columns.
    ///
    /// `num_rand_rows` indicates the number of rows at the end of the trace which will be
    /// overwritten with random values. This parameter is unused because last rows are just
    /// duplicates of the prior rows and thus can be safely overwritten.
    pub fn into_trace(mut self, trace_len: usize, num_rand_rows: usize) -> SysTrace {
        let clk = self.clk() as usize;
        // make sure that only the duplicate rows will be overwritten with random values
        assert!(clk + num_rand_rows <= trace_len, "target trace length too small");

        // complete the clk column by filling in all values after the last clock cycle. The values
        // in the clk column are equal to the index of the row in the trace table.
        self.clk_trace.resize(trace_len, ZERO);
        for (i, clk) in self.clk_trace.iter_mut().enumerate().skip(clk) {
            // converting from u32 is OK here because max trace length is 2^32
            *clk = Felt::from(i as u32);
        }

        // complete the ctx column by filling all values after the last clock cycle with ZEROs as
        // the last context must be zero context.
        debug_assert_eq!(0, self.ctx);
        self.ctx_trace.resize(trace_len, ZERO);

        // complete the fmp column by filling in all values after the last clock cycle with the
        // value in the column at the last clock cycle.
        let last_value = self.fmp_trace[clk];
        self.fmp_trace[clk..].fill(last_value);
        self.fmp_trace.resize(trace_len, last_value);

        // complete the in_syscall column by filling all values after the last clock cycle with
        // ZEROs as we must end the program in the root context which is not a SYSCALL
        debug_assert!(!self.in_syscall);
        self.in_syscall_trace.resize(trace_len, ZERO);

        let mut trace = vec![self.clk_trace, self.fmp_trace, self.ctx_trace, self.in_syscall_trace];

        // complete the fn hash columns by filling them with ZEROs as program execution must always
        // end in the root context.
        debug_assert_eq!(self.fn_hash, [ZERO; 4]);
        for mut column in self.fn_hash_trace.into_iter() {
            column.resize(trace_len, ZERO);
            trace.push(column);
        }

        trace.try_into().expect("failed to convert vector to array")
    }

    // UTILITY METHODS
    // --------------------------------------------------------------------------------------------

    /// Makes sure there is enough memory allocated for the trace to accommodate a new row.
    ///
    /// Trace length is doubled every time it needs to be increased.
    pub fn ensure_trace_capacity(&mut self) {
        let current_capacity = self.clk_trace.len();
        if self.clk + 1 >= current_capacity as u32 {
            let new_length = current_capacity * 2;
            self.clk_trace.resize(new_length, ZERO);
            self.ctx_trace.resize(new_length, ZERO);
            self.fmp_trace.resize(new_length, ZERO);
            self.in_syscall_trace.resize(new_length, ZERO);
            for column in self.fn_hash_trace.iter_mut() {
                column.resize(new_length, ZERO);
            }
        }
    }
}
