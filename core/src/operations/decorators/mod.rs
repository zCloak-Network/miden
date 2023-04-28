mod advice;
mod assembly_op;
use crate::utils::collections::Vec;
pub use advice::AdviceInjector;
pub use assembly_op::AssemblyOp;
use core::fmt;

// DECORATORS
// ================================================================================================

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Decorator {
    /// Pushes zero or more values onto the advice stack, as specified by the injector. This
    /// operation affects only the advice stack and has no effect on other VM components (e.g.
    /// operand stack, memory), and does not advance the VM clock.
    Advice(AdviceInjector),
    /// Adds information about the assembly instruction at a particular index
    /// (only applicable in debug mode)
    AsmOp(AssemblyOp),
}

impl fmt::Display for Decorator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Advice(injector) => write!(f, "advice({injector})"),
            Self::AsmOp(assembly_op) => {
                write!(f, "asmOp({}, {})", assembly_op.op(), assembly_op.num_cycles())
            }
        }
    }
}

/// Vector consisting of a tuple of operation index (within a span block) and decorator at that index
pub type DecoratorList = Vec<(usize, Decorator)>;

/// Iterator used to iterate through the decorator list of a span block
/// while executing operation batches of a span block.
pub struct DecoratorIterator<'a> {
    decorators: &'a DecoratorList,
    idx: usize,
}

impl<'a> DecoratorIterator<'a> {
    /// Returns a new instance of decorator iterator instantiated with the provided decorator list.
    pub fn new(decorators: &'a DecoratorList) -> Self {
        Self { decorators, idx: 0 }
    }

    /// Returns the next decorator at the specified position.
    /// - Returns the decorator if a decorator at the specified position exists and increments the internal pointer.
    /// - Returns None if no decorator is to be executed at the specified position.
    #[inline(always)]
    pub fn next(&mut self, pos: usize) -> Option<&Decorator> {
        if self.idx < self.decorators.len() && self.decorators[self.idx].0 == pos {
            self.idx += 1;
            Some(&self.decorators[self.idx - 1].1)
        } else {
            None
        }
    }
}
