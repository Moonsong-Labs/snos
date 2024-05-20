use std::collections::{HashMap, HashSet};

use blockifier::abi::abi_utils::get_fee_token_var_address;
use blockifier::block_context::BlockContext;
use blockifier::execution::contract_class::ContractClass::{V0, V1};
use blockifier::execution::contract_class::ContractClassV1;
use blockifier::state::cached_state::CachedState;
use blockifier::state::state_api::{State as _, StateReader};
use blockifier::test_utils::dict_state_reader::DictStateReader;
use blockifier::test_utils::CairoVersion;
use blockifier::transaction::objects::{FeeType, TransactionExecutionInfo};
use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use cairo_vm::Felt252;
use snos::config::{StarknetGeneralConfig, StarknetOsConfig, STORED_BLOCK_HASH_BUFFER};
use snos::execution::helper::ExecutionHelperWrapper;
use snos::io::input::{ContractState, StarknetOsInput, StorageCommitment};
use snos::io::InternalTransaction;
use snos::starknet::business_logic::utils::{write_compiled_class_fact, write_deprecated_compiled_class_fact};
use snos::storage::storage::{FactFetchingContext, HashFunctionType, Storage, StorageError};
use snos::storage::storage_utils::build_starknet_storage;
use snos::utils::felt_api2vm;
use starknet_api::state::ContractClass;
use starknet_api::core::{ClassHash, CompiledClassHash, ContractAddress, PatriciaKey};
use starknet_api::deprecated_contract_class::{
    ContractClass as DeprecatedCompiledClass, ContractClass as DeprecatedContractClass,
};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::{contract_address, patricia_key, stark_felt};
use starknet_crypto::FieldElement;

use crate::common::state::{ContractDeployment, DeprecatedContractDeployment, FeeContracts};
use crate::common::transaction_utils::to_felt252;

use super::state::TestState;

// TODO: move / organize, clean up types
/// Convert a starknet_api deprecated ContractClass to a cairo-vm ContractClass (v0 only).
/// Note that this makes a serialize -> deserialize pass, so it is not cheap!
pub fn deprecated_contract_class_api2vm(
    api_class: &starknet_api::deprecated_contract_class::ContractClass,
) -> serde_json::Result<blockifier::execution::contract_class::ContractClass> {
    let serialized = serde_json::to_string(&api_class)?;

    let vm_class_v0_inner: blockifier::execution::contract_class::ContractClassV0Inner =
        serde_json::from_str(serialized.as_str())?;

    let vm_class_v0 = blockifier::execution::contract_class::ContractClassV0(std::sync::Arc::new(vm_class_v0_inner));
    let vm_class = blockifier::execution::contract_class::ContractClass::V0(vm_class_v0);

    Ok(vm_class)
}

/// Convert a starknet_api ContractClass to a cairo-vm ContractClass (v1 only).
/// Note that this makes a serialize -> deserialize pass, so it is not cheap!
pub fn contract_class_cl2vm(
    cl_class: &CasmContractClass,
) -> Result<blockifier::execution::contract_class::ContractClass, cairo_vm::types::errors::program_errors::ProgramError> {
    let v1_class = ContractClassV1::try_from(cl_class.clone()).unwrap(); // TODO: type issue?
    Ok(v1_class.into())
}

fn stark_felt_from_bytes(bytes: Vec<u8>) -> StarkFelt {
    StarkFelt::new(bytes[..32].try_into().expect("Number is not 32-bytes"))
        .expect("Number is too large to be a felt 252")
}

/// Utility to fund an account.
/// Copied from Blockifier, but takes a DictStateReader directly.
pub fn fund_account(
    block_context: &BlockContext,
    account_address: ContractAddress,
    initial_balance: u128,
    dict_state_reader: &mut DictStateReader,
) {
    let storage_view = &mut dict_state_reader.storage_view;
    let balance_key = get_fee_token_var_address(account_address);
    for fee_type in [FeeType::Strk, FeeType::Eth] {
        storage_view.insert((block_context.fee_token_address(&fee_type), balance_key), stark_felt!(initial_balance));
    }
}

/// Returns test state used for all tests
pub async fn test_state<S, H>(
    block_context: &BlockContext,
    initial_balance_all_accounts: u128,
    erc20_class: DeprecatedCompiledClass,
    deprecated_contract_classes: &[(&str, &DeprecatedCompiledClass)],
    contract_classes: &[(&str, &CasmContractClass)],
    ffc: &mut FactFetchingContext<S, H>,
) -> Result<TestState, StorageError>
where
    S: Storage,
    H: HashFunctionType,
{
    // we use DictStateReader as a container for all the state we want to collect
    let mut state = DictStateReader::default();

    // Declare and deploy account and ERC20 contracts.
    let erc20_class_hash_bytes = write_deprecated_compiled_class_fact(erc20_class.clone(), ffc).await?;
    let erc20_class_hash = ClassHash(stark_felt_from_bytes(erc20_class_hash_bytes));

    state.class_hash_to_class.insert(erc20_class_hash, deprecated_contract_class_api2vm(&erc20_class).unwrap());
    state.address_to_class_hash.insert(block_context.fee_token_address(&FeeType::Eth), erc20_class_hash);
    state.address_to_class_hash.insert(block_context.fee_token_address(&FeeType::Strk), erc20_class_hash);

    let mut deployed_addresses = Vec::new();
    let mut deployed_deprecated_contract_classes = HashMap::new();
    deployed_deprecated_contract_classes.insert(erc20_class_hash, erc20_class.clone());

    let mut deployed_contract_classes = HashMap::new();
    let mut cairo0_contracts = HashMap::new();
    let mut cairo1_contracts = HashMap::new();

    // Deploy deprecated contracts
    for (name, contract) in deprecated_contract_classes {
        let class_hash_bytes = write_deprecated_compiled_class_fact((*contract).clone(), ffc).await?;
        let class_hash = ClassHash(stark_felt_from_bytes(class_hash_bytes));

        let vm_class = deprecated_contract_class_api2vm(contract).unwrap();
        state.class_hash_to_class.insert(class_hash, vm_class);

        let address = contract_address!(class_hash.0);
        state.address_to_class_hash.insert(address, class_hash);
        deployed_addresses.push(address);
        deployed_deprecated_contract_classes.insert(class_hash, (*contract).clone()); // TODO: remove

        cairo0_contracts.insert(name.to_string(), DeprecatedContractDeployment {
            class: (*contract).clone(),
            class_hash,
            address,
        });
    }

    // Deploy non-deprecated contracts
    for (name, contract) in contract_classes {
        let class_hash_bytes = write_compiled_class_fact((*contract).clone(), ffc).await?;
        let class_hash = ClassHash(stark_felt_from_bytes(class_hash_bytes));

        let vm_class = contract_class_cl2vm(contract).unwrap();
        state.class_hash_to_class.insert(class_hash, vm_class);

        let address = contract_address!(class_hash.0);
        state.address_to_class_hash.insert(address, class_hash);
        deployed_addresses.push(address);

        deployed_contract_classes.insert(class_hash, (*contract).clone());
        cairo1_contracts.insert(name.to_string(), ContractDeployment {
            class: (*contract).clone(),
            class_hash,
            address,
        });
    }

    let mut addresses: HashSet<ContractAddress> = Default::default();
    for address in state.address_to_class_hash.keys().chain(state.address_to_nonce.keys()) {
        addresses.insert(*address);
    }

    // fund the accounts.
    for address in addresses.iter() {
        fund_account(block_context, *address, initial_balance_all_accounts, &mut state);
    }

    println!("State dump:");
    println!(" - address_to_nonce:");
    for (k, v) in &state.address_to_nonce {
        println!("   - {:?} -> {:?}", k, v);
    }
    println!(" - address_to_class_hash:");
    for (k, v) in &state.address_to_class_hash {
        println!("   - {:?} -> {:?}", k, v);
    }
    println!(" - class_hash_to_class:");
    for (k, _v) in &state.class_hash_to_class {
        // println!("   - {:?} -> {:?}", k, v);
        println!("   - {:?} -> <omitted>", k);
    }
    println!(" - class_hash_to_compiled_class_hash:");
    for (k, _v) in &state.class_hash_to_compiled_class_hash {
        // println!("   - {:?} -> {:?}", k, v);
        println!("   - {:?} -> <omitted>", k);
    }

    Ok(TestState {
        cairo0_contracts,
        cairo1_contracts,
        fee_contracts: FeeContracts {
            erc20_contract: erc20_class,
            eth_fee_token_address: block_context.fee_token_address(&FeeType::Eth),
            strk_fee_token_address: block_context.fee_token_address(&FeeType::Strk),
        },
        blockifier_state: CachedState::from(state),
        deployed_addresses,
        contract_classes: deployed_contract_classes,
        deprecated_contract_classes: deployed_deprecated_contract_classes,
    })
}

/// Returns test state for cairo1-based tests
pub async fn test_state_cairo1<S, H>(
    block_context: &BlockContext,
    initial_balance_all_accounts: u128,
    erc20_class: &DeprecatedContractClass,
    contract_classes: &[&CasmContractClass],
    ffc: &mut FactFetchingContext<S, H>,
) -> Result<
    (CachedState<DictStateReader>, Vec<ContractAddress>, HashMap<ClassHash, DeprecatedContractClass>),
    StorageError,
>
where
    S: Storage,
    H: HashFunctionType,
{
    // we use DictStateReader as a container for all the state we want to collect
    let mut state = DictStateReader::default();

    // Declare and deploy account and ERC20 contracts.
    let erc20_class_hash_bytes = write_deprecated_compiled_class_fact(erc20_class.clone(), ffc).await?;
    let erc20_class_hash = ClassHash(stark_felt_from_bytes(erc20_class_hash_bytes));

    state.class_hash_to_class.insert(erc20_class_hash, deprecated_contract_class_api2vm(erc20_class).unwrap());
    state.address_to_class_hash.insert(block_context.fee_token_address(&FeeType::Eth), erc20_class_hash);
    state.address_to_class_hash.insert(block_context.fee_token_address(&FeeType::Strk), erc20_class_hash);

    let mut deployed_addresses = Vec::new();
    let mut deprecated_contract_classes = HashMap::new();
    deprecated_contract_classes.insert(erc20_class_hash, (*erc20_class).clone());

    // Set up the rest of the requested contracts.
    for contract in contract_classes {
        println!("processing contract...");
        let class_hash_bytes = write_compiled_class_fact((*contract).clone(), ffc).await?;
        let class_hash = ClassHash(stark_felt_from_bytes(class_hash_bytes));
        println!(" - class_hash: {:?}", class_hash);

        let vm_class = contract_class_cl2vm(contract).unwrap();
        state.class_hash_to_class.insert(class_hash, vm_class);

        // TODO: review -- this just seems to be generating a random address based on our seed
        // so it should just need a unique input, which our class hash should give us
        let address = contract_address!(class_hash.0);
        state.address_to_class_hash.insert(address, class_hash);
        println!(" - address: {:?}", address);
        deployed_addresses.push(address);
    }

    let mut addresses: HashSet<ContractAddress> = Default::default();
    for address in state.address_to_class_hash.keys().chain(state.address_to_nonce.keys()) {
        addresses.insert(*address);
    }

    // fund the accounts.
    for address in addresses.iter() {
        fund_account(block_context, *address, initial_balance_all_accounts, &mut state);
    }

    println!("State dump:");
    println!(" - address_to_nonce:");
    for (k, v) in &state.address_to_nonce {
        println!("   - {:?} -> {:?}", k, v);
    }
    println!(" - address_to_class_hash:");
    for (k, v) in &state.address_to_class_hash {
        println!("   - {:?} -> {:?}", k, v);
    }
    println!(" - class_hash_to_class:");
    for (k, _v) in &state.class_hash_to_class {
        // println!("   - {:?} -> {:?}", k, v);
        println!("   - {:?} -> <omitted>", k);
    }
    println!(" - class_hash_to_compiled_class_hash:");
    for (k, _v) in &state.class_hash_to_compiled_class_hash {
        // println!("   - {:?} -> {:?}", k, v);
        println!("   - {:?} -> <omitted>", k);
    }

    Ok((CachedState::from(state), deployed_addresses, deprecated_contract_classes))
}

pub async fn os_hints(
    block_context: &BlockContext,
    mut blockifier_state: CachedState<DictStateReader>,
    transactions: Vec<InternalTransaction>,
    tx_execution_infos: Vec<TransactionExecutionInfo>,
    compiled_classes: HashMap<ClassHash, CasmContractClass>,
    deprecated_compiled_classes: HashMap<ClassHash, DeprecatedContractClass>,
) -> (StarknetOsInput, ExecutionHelperWrapper) {
    let deployed_addresses = blockifier_state.to_state_diff().address_to_class_hash;
    let initial_addresses = blockifier_state.state.address_to_class_hash.keys().cloned().collect::<HashSet<_>>();
    let addresses = deployed_addresses.keys().cloned().chain(initial_addresses);

    let mut contracts: HashMap<Felt252, ContractState> = addresses
        .map(|address| {
            // os expects the contract hash to be 0 for just deployed contracts
            let contract_hash = if deployed_addresses.contains_key(&address) {
                Felt252::ZERO
            } else {
                to_felt252(&blockifier_state.get_class_hash_at(address).unwrap().0)
            };
            let contract_state = ContractState {
                contract_hash,
                storage_commitment_tree: StorageCommitment::default(), // TODO
                nonce: 0.into(),                                       // TODO
            };
            (to_felt252(address.0.key()), contract_state)
        })
        .collect();

    let mut class_hash_to_compiled_class_hash: HashMap<Felt252, Felt252> = Default::default();

    for c in contracts.keys() {
        let address = ContractAddress::try_from(StarkHash::new(c.to_bytes_be()).unwrap()).unwrap();
        let class_hash = blockifier_state.get_class_hash_at(address).unwrap();
        let blockifier_class = blockifier_state.get_compiled_contract_class(class_hash).unwrap();
        match blockifier_class {
            V0(_) => {} // deprecated_compiled_classes are passed in by caller
            V1(_) => {
                let class = compiled_classes.get(&class_hash).expect(format!("No class given for {:?}", class_hash).as_str());
                let compiled_class_hash = class.compiled_class_hash();
                class_hash_to_compiled_class_hash
                    .insert(to_felt252(&class_hash.0), Felt252::from_bytes_be(&compiled_class_hash.to_be_bytes()));
            }
        };
    }

    contracts.insert(Felt252::from(0), ContractState::default());
    contracts.insert(Felt252::from(1), ContractState::default());

    println!("contracts: {:?}\ndeprecated_compiled_classes: {:?}", contracts.len(), deprecated_compiled_classes.len());

    println!("contracts to class_hash");
    for (a, c) in &contracts {
        println!("\t{} -> {}", a, c.contract_hash);
    }

    println!("deprecated classes");
    for (c, _) in &deprecated_compiled_classes {
        println!("\t{}", c);
    }

    println!("classes");
    for (c, _) in &compiled_classes {
        println!("\t{}", c);
    }

    // for h in deprecated_compiled_classes.keys() {
    //     class_hash_to_compiled_class_hash.insert(h.clone(), h.clone());
    // }

    // for (h, c) in compiled_classes.iter() {
    //     class_hash_to_compiled_class_hash
    //         .insert(h.clone(), Felt252::from_bytes_be(&c.compiled_class_hash().to_be_bytes()));
    // }

    println!("class_hash to compiled_class_hash");
    for (ch, cch) in &class_hash_to_compiled_class_hash {
        println!("\t{} -> {}", ch, cch);
    }

    let default_general_config = StarknetGeneralConfig::default();

    let general_config = StarknetGeneralConfig {
        starknet_os_config: StarknetOsConfig {
            chain_id: default_general_config.starknet_os_config.chain_id,
            fee_token_address: block_context.fee_token_addresses.strk_fee_token_address,
            deprecated_fee_token_address: block_context.fee_token_addresses.eth_fee_token_address,
        },
        ..default_general_config
    };

    let deprecated_compiled_classes: HashMap<_, _> =
        deprecated_compiled_classes.into_iter().map(|(k, v)| (felt_api2vm(k.0), v)).collect();

    let compiled_classes: HashMap<_, _> =
        compiled_classes.into_iter().map(|(k, v)| (felt_api2vm(k.0), v)).collect();

    let os_input = StarknetOsInput {
        contract_state_commitment_info: Default::default(),
        contract_class_commitment_info: Default::default(),
        deprecated_compiled_classes,
        compiled_classes,
        compiled_class_visited_pcs: Default::default(),
        contracts,
        class_hash_to_compiled_class_hash,
        general_config,
        transactions,
        block_hash: Default::default(),
    };

    // Convert the Blockifier storage into an OS-compatible one
    let contract_storage_map = build_starknet_storage(&mut blockifier_state).await;

    let execution_helper = ExecutionHelperWrapper::new(
        contract_storage_map,
        tx_execution_infos,
        &block_context,
        (Felt252::from(block_context.block_number.0 - STORED_BLOCK_HASH_BUFFER), Felt252::from(66_u64)),
    );

    (os_input, execution_helper)
}
