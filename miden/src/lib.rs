#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

// EXPORTS
// ================================================================================================

pub use assembly::{Assembler, AssemblyError, ParsingError};
pub use processor::{
    crypto, execute, execute_iter, utils, AdviceInputs, AdviceProvider, AsmOpInfo, ExecutionError,
    ExecutionTrace, Kernel, MemAdviceProvider, Operation, ProgramInfo, StackInputs, VmState,
    VmStateIterator,
};
pub use prover::{
    math, prove, Digest, ExecutionProof, FieldExtension, HashFunction, InputError, Program,
    ProofOptions, StackOutputs, StarkProof, Word,
};
pub use verifier::{verify, VerificationError};
use serde::{Deserialize, Serialize};
extern crate wasm_bindgen;
use vm_core::Felt;
use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;

#[derive(Debug, Serialize, Deserialize)]
pub struct NormalInput {
    stack_inputs: StackInputs,
    advice_provider: MemAdviceProvider,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VMResult {
    outputs: StackOutputs,
    starkproof: ExecutionProof,
}


#[wasm_bindgen]
pub fn execute_zk_program(
    program_code: String,
    program_inputs: String,
) -> String {

    let options = ProofOptions::with_96_bit_security();


    let assembler = Assembler::default()
    .with_library(&stdlib::StdLibrary::default()).unwrap();

    
    let program = assembler.compile(&program_code).unwrap();


    let inputs_slice: &str = &program_inputs[..];

    let inputs: NormalInput = serde_json::from_str(inputs_slice).unwrap();

    let res = prove(&program, inputs.stack_inputs, inputs.advice_provider, options);

    assert!(res.is_ok(), "The proof generation fails: {:?}", res);

    let (outputs, proof) = res.unwrap();

    let result = VMResult {
        outputs,
        starkproof: proof,
    };

    let final_result = serde_json::to_string(&result).unwrap();
    return final_result;
}


#[wasm_bindgen]
pub fn generate_program_hash(program_in_assembly: String) -> String {
    let assembler = Assembler::default()
    .with_library(&stdlib::StdLibrary::default()).unwrap();
    let program = assembler.compile(&program_in_assembly).unwrap();
    use vm_core::utils::Serializable;
    let program_hash = program.hash().to_bytes();
    let ph = hex::encode(program_hash);
    return ph;
}

#[wasm_bindgen]
pub fn convert_stackinputs(
    stack_init: String,
    advice_tape: String,
) -> String {
    let mut stack_inita = vec![];
    let mut advice_tapea = vec![];
    if stack_init.len() != 0 {
        let stack_init: Vec<&str> = stack_init.split(',').collect();
        stack_inita = stack_init
            .iter()
            .map(|stack_init| Felt::new(stack_init.parse::<u64>().unwrap()))
            .collect();
    };

    if advice_tape.len() != 0 {
        let advice_tape: Vec<&str> = advice_tape.split(',').collect();
        advice_tapea = advice_tape
            .iter()
            .map(|advice_tape| Felt::new(advice_tape.parse::<u64>().unwrap()))
            .collect();
    };

    let stack_input: StackInputs = StackInputs::new(stack_inita);
    let advice_inputs = AdviceInputs::default().with_stack(advice_tapea);

    let mem_advice_provider: MemAdviceProvider = MemAdviceProvider::from(advice_inputs);
    let inputs = NormalInput {
        stack_inputs: stack_input,
        advice_provider: mem_advice_provider
    };
  
    let serialized = serde_json::to_string(&inputs).unwrap();
    return serialized;
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
