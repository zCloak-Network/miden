#![cfg_attr(not(feature = "std"), no_std)]

// EXPORTS
// ================================================================================================

pub use air::{FieldExtension, HashFunction, ProofOptions};
pub use assembly::{Assembler, AssemblyError};
use crypto::hash::rescue::rp64_256::ElementDigest;
pub use prover::{prove, StarkProof};
use serde::{Deserialize, Serialize};
pub use verifier::{verify, VerificationError};
use vm_core::utils::SliceReader;
pub use vm_core::{
    chiplets::hasher::Digest,
    errors::{AdviceSetError, InputError},
    AdviceSet, Program, ProgramInputs,
};
use winter_utils::{Deserializable, Serializable};
extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;

#[derive(Debug, Serialize, Deserialize)]
pub struct VMResult {
    outputs: Vec<u64>,
    starkproof: StarkProof,
}

#[wasm_bindgen]
pub fn execute(
    program: String,
    inputs: String,
    num_stack_outputs: usize,
) -> String {
    let options = &ProofOptions::with_96_bit_security();
    let assembler = Assembler::new(true);
    let program = assembler.compile(&program).unwrap();

    let inputs_slice: &str = &inputs[..];

    let inputs: ProgramInputs = serde_json::from_str(inputs_slice).unwrap();

    let res = prove(&program, &inputs, num_stack_outputs, options);
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
pub fn generate_program_hash(program_in_assembly: String) -> Vec<u8> {
    let assembler = Assembler::new(true);
    let program = assembler.compile(&program_in_assembly).unwrap();
    use vm_core::utils::Serializable;
    let program_hash = program.hash().to_bytes();
    return program_hash;
}

#[wasm_bindgen]
pub fn program_verify(
    program_hash: Vec<u8>,
    stack_inputs: Vec<u64>,
    stack_outputs: Vec<u64>,
    proof: String,
) -> bool {
    // the program hash should be a Vec with 32 elements
    assert_eq!(32, program_hash.len());
    let mut reader = SliceReader::new(&program_hash);
    let program_hash_digest = ElementDigest::read_from(&mut reader).unwrap();
    let stark_proof: StarkProof = serde_json::from_str(&proof).unwrap();
    let result = verify(
        program_hash_digest,
        &stack_inputs,
        &stack_outputs,
        stark_proof,
    )
    .is_ok();
    return result;
}

#[wasm_bindgen]
pub fn output_inputs_string(
    stack_init: String,
    advice_tape: String,
    _advice_sets: String,
) -> String {
    let mut stack_inita = vec![];
    let mut advice_tapea = vec![];
    if stack_init.len() != 0 {
        let stack_init: Vec<&str> = stack_init.split(',').collect();
        stack_inita = stack_init
            .iter()
            .map(|stack_init| stack_init.parse::<u64>().unwrap())
            .collect();
    };

    if advice_tape.len() != 0 {
        let advice_tape: Vec<&str> = advice_tape.split(',').collect();
        advice_tapea = advice_tape
            .iter()
            .map(|advice_tape| advice_tape.parse::<u64>().unwrap())
            .collect();
    };
    let advice_setsa = Vec::new();

    let inputs = ProgramInputs::new(&stack_inita, &advice_tapea, advice_setsa);
    assert!(
        inputs.is_ok(),
        "The input initialization failed, please check your inputs."
    );
    let serialized = serde_json::to_string(&inputs.unwrap()).unwrap();
    return serialized;
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
