use std::path::{Path, PathBuf};

use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use cairo_lang_starknet_classes::contract_class::ContractClass;

use crate::common::contract_fixtures::load_cairo1_contract;

fn get_contract_path(contract_name: &str) -> PathBuf {
    let filename = format!("{contract_name}.sierra");
    Path::new("openzeppelin").join("compiled").join(filename)
}

pub(crate) fn load_upgradable_account() -> (String, ContractClass, CasmContractClass) {
    let account_name = "upgradable_account";
    let contract_path = get_contract_path(account_name);
    let (sierra_class, casm_class) = load_cairo1_contract(&contract_path);

    (account_name.to_string(), sierra_class, casm_class)
}
