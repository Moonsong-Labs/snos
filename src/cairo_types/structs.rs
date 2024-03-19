use cairo_type_derive::FieldOffsetGetters;
use cairo_vm::Felt252;

#[allow(unused)]
#[derive(FieldOffsetGetters)]
pub struct ExecutionContext {
    pub entry_point_type: Felt252,
    pub class_hash: Felt252,
    pub calldata_size: Felt252,
    pub calldata: Felt252,
    pub execution_info: Felt252,
    pub deprecated_tx_info: Felt252,
}

#[allow(unused)]
#[derive(FieldOffsetGetters)]
pub struct CompiledClassFact {
    pub hash: Felt252,
    pub compiled_class: Felt252,
}

#[allow(unused)]
#[derive(FieldOffsetGetters)]
pub struct CompiledClass {
    compiled_class_version: Felt252,
    n_external_functions: Felt252,
    external_functions: Felt252,
    n_l1_handlers: Felt252,
    l1_handlers: Felt252,
    n_constructors: Felt252,
    constructors: Felt252,
    bytecode_length: Felt252,
    bytecode_ptr: Felt252,
}
