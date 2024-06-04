pub mod scopes {
    pub const BLOCK_CONTEXT: &str = "block_context";
    pub const BYTECODE_SEGMENT_STRUCTURE: &str = "bytecode_segment_structure";
    pub const BYTECODE_SEGMENTS: &str = "bytecode_segments";
    pub const CASE: &str = "case";
    pub const COMMITMENT_INFO: &str = "commitment_info";
    pub const COMMITMENT_INFO_BY_ADDRESS: &str = "commitment_info_by_address";
    pub const COMPILED_CLASS: &str = "compiled_class";
    pub const COMPILED_CLASS_FACTS: &str = "compiled_class_facts";
    pub const COMPILED_CLASS_VISITED_PCS: &str = "compiled_class_visited_pcs";
    pub const COMPILED_CLASS_HASH: &str = "compiled_class_hash";
    pub const DEPRECATED_CLASS_HASHES: &str = "__deprecated_class_hashes";
    pub const DEPRECATED_SYSCALL_HANDLER: &str = "deprecated_syscall_handler";
    pub const DESCEND: &str = "descend";

    pub const DESCENT_MAP: &str = "descent_map";
    pub const DICT_MANAGER: &str = "dict_manager";
    pub const EXECUTION_HELPER: &str = "execution_helper";
    pub const INITIAL_DICT: &str = "initial_dict";
    pub const IS_DEPRECATED: &str = "is_deprecated";
    pub const N_SELECTED_BUILTINS: &str = "n_selected_builtins";
    pub const NODE: &str = "node";
    pub const LEFT_CHILD: &str = "left_child";
    pub const OS_INPUT: &str = "os_input";
    pub const PATRICIA_SKIP_VALIDATION_RUNNER: &str = "__patricia_skip_validation_runner";
    pub const PATRICIA_TREE_MODE: &str = "patricia_tree_mode";
    pub const PREIMAGE: &str = "preimage";
    pub const RIGHT_CHILD: &str = "right_child";
    pub const SYSCALL_HANDLER: &str = "syscall_handler";
    pub const TRANSACTIONS: &str = "transactions";
    pub const TX: &str = "tx";
    pub const VALUE: &str = "value";
}

pub mod ids {
    pub const ADDITIONAL_DATA: &str = "additional_data";
    pub const ALL_ENCODINGS: &str = "all_encodings";
    pub const BIT: &str = "bit";
    pub const BUILTIN_PARAMS: &str = "builtin_params";
    pub const BUILTIN_PTRS: &str = "builtin_ptrs";
    pub const CALL_RESPONSE: &str = "call_response";
    pub const CALLDATA: &str = "calldata";
    pub const CHILD_BIT: &str = "child_bit";
    pub const CLASS_HASH_PTR: &str = "class_hash_ptr";
    pub const COMPILED_CLASS: &str = "compiled_class";
    pub const COMPILED_CLASS_FACT: &str = "compiled_class_fact";
    pub const COMPILED_CLASS_FACTS: &str = "compiled_class_facts";
    pub const COMPILED_CLASS_VISITED_PCS: &str = "compiled_class_visited_pcs";
    pub const COMPILED_CLASS_HASH: &str = "compiled_class_hash";
    pub const CONSTRUCTOR_CALLDATA: &str = "constructor_calldata";
    pub const CONSTRUCTOR_CALLDATA_SIZE: &str = "constructor_calldata_size";
    pub const CONTRACT_ADDRESS: &str = "contract_address";
    pub const CONTRACT_STATE_CHANGES: &str = "contract_state_changes";
    pub const CURRENT_BLOCK_NUMBER: &str = "current_block_number";
    pub const CURRENT_HASH: &str = "current_hash";
    pub const DA_START: &str = "da_start";
    pub const DATA_TO_HASH: &str = "data_to_hash";
    pub const DEPRECATED_TX_INFO: &str = "deprecated_tx_info";
    pub const DESCEND: &str = "descend";
    pub const DEST_PTR: &str = "dest_ptr";
    pub const EDGE: &str = "edge";
    pub const ELEMENTS: &str = "elements";
    pub const ELEMENTS_END: &str = "elements_end";
    pub const ENTRY_POINT_RETURN_VALUES: &str = "entry_point_return_values";
    pub const EXECUTION_CONTEXT: &str = "execution_context";
    pub const FINAL_CONTRACT_STATE_ROOT: &str = "final_contract_state_root";
    pub const FINAL_ROOT: &str = "final_root";
    pub const HASH_PTR: &str = "hash_ptr";
    pub const INITIAL_GAS: &str = "initial_gas";
    pub const IS_LEAF: &str = "is_leaf";
    pub const HEIGHT: &str = "height";
    pub const INITIAL_CARRIED_OUTPUTS: &str = "initial_carried_outputs";
    pub const INITIAL_CONTRACT_STATE_ROOT: &str = "initial_contract_state_root";
    pub const INITIAL_ROOT: &str = "initial_root";
    pub const IS_ON_CURVE: &str = "is_on_curve";
    pub const USE_KZG_DA: &str = "use_kzg_da";
    pub const LENGTH: &str = "length";
    pub const LOW: &str = "low";
    pub const MAX_FEE: &str = "max_fee";
    pub const N: &str = "n";
    pub const N_BUILTINS: &str = "n_builtins";
    pub const N_COMPILED_CLASS_FACTS: &str = "n_compiled_class_facts";
    pub const N_SELECTED_BUILTINS: &str = "n_selected_builtins";
    pub const N_UPDATES: &str = "n_updates";
    pub const NEW_LENGTH: &str = "new_length";
    pub const NEW_ROOT: &str = "new_root";
    pub const NEW_STATE_ENTRY: &str = "new_state_entry";
    pub const NODE: &str = "node";
    pub const OLD_BLOCK_HASH: &str = "old_block_hash";
    pub const OLD_BLOCK_NUMBER: &str = "old_block_number";
    pub const OS_CONTEXT: &str = "os_context";
    pub const OUTPUT_PTR: &str = "output_ptr";
    pub const PATH: &str = "path";
    pub const PREV_ROOT: &str = "prev_root";
    pub const PREV_VALUE: &str = "prev_value";
    pub const REQUEST: &str = "request";
    pub const REQUEST_BLOCK_NUMBER: &str = "request_block_number";
    pub const REQUIRED_GAS: &str = "required_gas";
    pub const RES: &str = "res";
    pub const RESOURCE_BOUNDS: &str = "resource_bounds";
    pub const RESPONSE: &str = "response";
    pub const RETDATA: &str = "retdata";
    pub const RETDATA_SIZE: &str = "retdata_size";
    pub const RETURN_BUILTIN_PTRS: &str = "return_builtin_ptrs";
    pub const SECP_P: &str = "SECP_P";
    pub const SELECT_BUILTIN: &str = "select_builtin";
    pub const SELECTED_ENCODINGS: &str = "selected_encodings";
    pub const SELECTED_PTRS: &str = "selected_ptrs";
    pub const SELECTOR: &str = "selector";
    pub const SENDER_ADDRESS: &str = "sender_address";
    pub const SIBLINGS: &str = "siblings";
    pub const SIGNATURE_LEN: &str = "signature_len";
    pub const SIGNATURE_START: &str = "signature_start";
    pub const SRC_PTR: &str = "src_ptr";
    pub const STATE_ENTRY: &str = "state_entry";
    pub const STATE_UPDATES_START: &str = "state_updates_start";
    pub const SYSCALL_PTR: &str = "syscall_ptr";
    pub const TRANSACTION_HASH: &str = "transaction_hash";
    pub const TX_EXECUTION_CONTEXT: &str = "tx_execution_context";
    pub const TX_INFO: &str = "tx_info";
    pub const TX_TYPE: &str = "tx_type";
    pub const TX_VERSION: &str = "tx_version";
    pub const UPDATE_PTR: &str = "update_ptr";
    pub const VALIDATE_DECLARE_EXECUTION_CONTEXT: &str = "validate_declare_execution_context";
    pub const VALUE: &str = "value";
    pub const WORD: &str = "word";
    pub const Y: &str = "y";
    pub const Y_SQUARE_INT: &str = "y_square_int";
}

pub mod constants {
    pub const BASE: &str = "starkware.starknet.core.os.data_availability.bls_field.BASE";
    pub const BLOCK_HASH_CONTRACT_ADDRESS: &str = "starkware.starknet.core.os.constants.BLOCK_HASH_CONTRACT_ADDRESS";
    pub const MERKLE_HEIGHT: &str = "starkware.starknet.core.os.state.commitment.MERKLE_HEIGHT";
    pub const STORED_BLOCK_HASH_BUFFER: &str = "starkware.starknet.core.os.constants.STORED_BLOCK_HASH_BUFFER";
    pub const VALIDATED: &str = "starkware.starknet.core.os.constants.VALIDATED";
}
