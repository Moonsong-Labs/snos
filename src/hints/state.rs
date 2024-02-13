use std::collections::HashMap;

use cairo_vm::hint_processor::builtin_hint_processor::hint_utils::{get_integer_from_var_name, insert_value_from_var_name};
use cairo_vm::hint_processor::hint_processor_definition::HintReference;
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::vm_core::VirtualMachine;
use cairo_vm::Felt252;
use indoc::indoc;

use crate::io::input::StarknetOsInput;
use crate::hints::vars;

pub const SET_PREIMAGE_FOR_STATE_COMMITMENTS: &str = indoc! {r#"
	ids.initial_root = os_input.contract_state_commitment_info.previous_root
	ids.final_root = os_input.contract_state_commitment_info.updated_root
	preimage = {
	    int(root): children
	    for root, children in os_input.contract_state_commitment_info.commitment_facts.items()
	}
	assert os_input.contract_state_commitment_info.tree_height == ids.MERKLE_HEIGHT"#
};
pub fn set_preimage_for_state_commitments(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let os_input = exec_scopes.get::<StarknetOsInput>(vars::scopes::OS_INPUT)?;
    insert_value_from_var_name(vars::ids::INITIAL_ROOT, os_input.contract_state_commitment_info.previous_root, vm, ids_data, ap_tracking)?;
    insert_value_from_var_name(vars::ids::FINAL_ROOT, os_input.contract_state_commitment_info.updated_root, vm, ids_data, ap_tracking)?;

    let preimage = os_input.contract_state_commitment_info.commitment_facts;
    exec_scopes.insert_value(vars::scopes::PREIMAGE, preimage);

    let merkle_height = get_integer_from_var_name(vars::ids::MERKLE_HEIGHT, vm, ids_data, ap_tracking)?
        .into_owned();
    let tree_height: Felt252 = os_input.contract_state_commitment_info.tree_height.into();
    assert_eq!(tree_height, merkle_height);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::io::input::CommitmentInfo;

    use super::*;

    use rstest::{fixture, rstest};

    #[fixture]
    fn os_input() -> StarknetOsInput {
        StarknetOsInput {
            contract_state_commitment_info: CommitmentInfo {
                previous_root: 1_usize.into(),
                updated_root: 2_usize.into(),
                tree_height: 251_usize.into(),
                commitment_facts: Default::default(),
            },
            contract_class_commitment_info: CommitmentInfo {
                previous_root: 11_usize.into(),
                updated_root: 12_usize.into(),
                tree_height: 251_usize.into(),
                commitment_facts: Default::default(),
            },
            deprecated_compiled_classes: Default::default(),
            compiled_classes: Default::default(),
            contracts: Default::default(),
            class_hash_to_compiled_class_hash: Default::default(),
            general_config: Default::default(),
            transactions: Default::default(),
            block_hash: Default::default(),
        }
    }

    #[rstest]
    fn test_set_preimage_for_state_commitments(os_input: StarknetOsInput) {
        let mut vm = VirtualMachine::new(false);
        vm.add_memory_segment();
        vm.add_memory_segment();
        vm.add_memory_segment();
        vm.set_fp(3);

        let ap_tracking = ApTracking::new();
        let constants = HashMap::new();

        let ids_data = HashMap::from([
            (vars::ids::INITIAL_ROOT.to_string(), HintReference::new_simple(-3)),
            (vars::ids::FINAL_ROOT.to_string(), HintReference::new_simple(-2)),
            (vars::ids::MERKLE_HEIGHT.to_string(), HintReference::new_simple(-1)),
        ]);
        insert_value_from_var_name(vars::ids::MERKLE_HEIGHT, 251_usize, &mut vm, &ids_data, &ap_tracking)
            .expect("Couldn't insert 252 into ids.MERKLE_HEIGHT");

        let mut exec_scopes: ExecutionScopes = Default::default();
        exec_scopes.insert_value(vars::scopes::OS_INPUT, os_input);

        set_preimage_for_state_commitments(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking, &constants).unwrap();

        assert_eq!(
            get_integer_from_var_name(vars::ids::INITIAL_ROOT, &vm, &ids_data, &ap_tracking).unwrap().into_owned(),
            1_usize.into()
        );
        assert_eq!(
            get_integer_from_var_name(vars::ids::FINAL_ROOT, &vm, &ids_data, &ap_tracking).unwrap().into_owned(),
            2_usize.into()
        );
        // TODO: test preimage more thoroughly
        assert!(exec_scopes.get::<HashMap<Felt252, Vec<Felt252>>>(vars::scopes::PREIMAGE).is_ok());
    }
}
