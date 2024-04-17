use std::collections::HashMap;

use cairo_vm::hint_processor::builtin_hint_processor::hint_utils::{
    get_integer_from_var_name, get_ptr_from_var_name, get_relocatable_from_var_name, insert_value_into_ap,
};
use cairo_vm::hint_processor::hint_processor_definition::HintReference;
use cairo_vm::hint_processor::hint_processor_utils::felt_to_usize;
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::vm_core::VirtualMachine;
use cairo_vm::Felt252;
use indoc::indoc;

use crate::execution::helper::ExecutionHelperWrapper;
use crate::hints::vars;

pub const WRITE_ZKG_COMMITMENT_ADDRESS: &str = indoc! {r#"
    execution_helper.store_da_segment(
        da_segment=memory.get_range_as_ints(addr=ids.state_updates_start, size=ids.da_size)
    )
    segments.write_arg(
        ids.kzg_commitment.address_,
        execution_helper.polynomial_coefficients_to_kzg_commitment_callback(
            execution_helper.da_segment
        )
    )"#
};

pub fn write_zkg_commitment_address(
    vm: &mut VirtualMachine,
    exec_scopes: &mut ExecutionScopes,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
    _constants: &HashMap<String, Felt252>,
) -> Result<(), HintError> {
    let mut execution_helper: ExecutionHelperWrapper = exec_scopes.get(vars::scopes::EXECUTION_HELPER)?;

    let state_updates_start = get_ptr_from_var_name(vars::ids::STATE_UPDATES_START, vm, ids_data, ap_tracking)?;
    let da_size_felt = get_integer_from_var_name(vars::ids::DA_START, vm, ids_data, ap_tracking)?;
    let da_size = felt_to_usize(&da_size_felt)?;

    let da_segment: Vec<_> =
        vm.get_integer_range(state_updates_start, da_size)?.into_iter().map(|x| x.into_owned()).collect();

    execution_helper.store_da_segment(da_segment)?;

    //     segments.write_arg(
    //         ids.kzg_commitment.address_,
    //         execution_helper.polynomial_coefficients_to_kzg_commitment_callback(
    //             execution_helper.da_segment
    //         )
    //     )"#
    let kzg_commitment_address = get_relocatable_from_var_name(vars::ids::KZG_COMMITMENT, vm, ids_data, ap_tracking)?;

    Ok(())
}
