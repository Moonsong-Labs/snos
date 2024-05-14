use cairo_vm::types::program::Program;
use starknet_api::deprecated_contract_class::ContractClass as DeprecatedCompiledClass;

use crate::storage::storage::HashFunctionType;

const SNOS_BUILD_DIR: &str = "../../../../../build";
const HASHER_PROGRAM_PATH: &str = concat!(SNOS_BUILD_DIR, "deprecated_compiled_class.json");
const HASHER_PROGRAM: &[u8] = include_bytes!(HASHER_PROGRAM_PATH);

pub fn compute_deprecated_class_hash<H: HashFunctionType>(contract_class: &DeprecatedCompiledClass) -> Vec<u8> {
    // TODO: the Python implementation has a caching mechanism, do we care?
    compute_deprecated_class_hash_inner(contract_class)
}

fn compute_deprecated_class_hash_inner<H: HashFunctionType>(contract_class: &DeprecatedCompiledClass) -> Vec<u8> {
    let hasher_program =
        Program::from_bytes(HASHER_PROGRAM, Some("main")).expect("Loading the Cairo 0 hasher program failed");

    let compiled_class_struct = get_deprecated_contract_class_struct(hasher_program.identifiers, contract_class);

    todo!()
}

fn get_deprecated_contract_class_struct(identifiers: _, contract_class: DeprecatedCompiledClass) -> 