use std::collections::HashMap;

use blockifier::block_context::BlockContext;
use blockifier::execution::contract_class::ContractClass::{V0, V1};
use blockifier::state::cached_state::CachedState;
use blockifier::state::state_api::{State, StateReader};
use blockifier::test_utils::contracts::FeatureContract;
use blockifier::test_utils::dict_state_reader::DictStateReader;
use blockifier::test_utils::initial_test_state::fund_account;
use blockifier::test_utils::CairoVersion;
use blockifier::transaction::objects::{FeeType, TransactionExecutionInfo};
use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use cairo_vm::Felt252;
use num_bigint::BigUint;
use snos::config::{StarknetGeneralConfig, StarknetOsConfig, BLOCK_HASH_CONTRACT_ADDRESS, STORED_BLOCK_HASH_BUFFER};
use snos::crypto::pedersen::PedersenHash;
use snos::execution::helper::ExecutionHelperWrapper;
use snos::io::input::StarknetOsInput;
use snos::io::InternalTransaction;
use snos::starknet::business_logic::fact_state::contract_state_objects::ContractState;
use snos::starknet::business_logic::fact_state::state::SharedState;
use snos::starknet::starknet_storage::execute_coroutine_threadsafe;
use snos::starkware_utils::commitment_tree::patricia_tree::patricia_tree::PatriciaTree;
use snos::storage::dict_storage::DictStorage;
use snos::storage::storage::FactFetchingContext;
use snos::storage::storage_utils::build_starknet_storage;
use snos::utils::{felt_api2vm, felt_vm2api};
use starknet_api::core::{ClassHash, CompiledClassHash, ContractAddress, PatriciaKey};
use starknet_api::deprecated_contract_class::ContractClass as DeprecatedContractClass;
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::stark_felt;
use starknet_api::state::StorageKey;
use starknet_crypto::FieldElement;

use crate::common::transaction_utils::to_felt252;

pub fn deprecated_compiled_class(class_hash: ClassHash) -> DeprecatedContractClass {
    let variants = vec![
        FeatureContract::AccountWithLongValidate(CairoVersion::Cairo0),
        FeatureContract::AccountWithoutValidations(CairoVersion::Cairo0),
        FeatureContract::ERC20,
        FeatureContract::Empty(CairoVersion::Cairo0),
        FeatureContract::FaultyAccount(CairoVersion::Cairo0),
        // LegacyTestContract,
        FeatureContract::SecurityTests,
        FeatureContract::TestContract(CairoVersion::Cairo0),
    ];

    for c in variants {
        if ClassHash(override_class_hash(&c)) == class_hash {
            let result: Result<DeprecatedContractClass, serde_json::Error> =
                serde_json::from_str(c.get_raw_class().as_str());
            return result.unwrap();
        }
    }
    panic!("No deprecated class found for hash: {:?}", class_hash);
}

pub fn compiled_class(class_hash: ClassHash) -> CasmContractClass {
    let variants = vec![
        FeatureContract::AccountWithLongValidate(CairoVersion::Cairo1),
        FeatureContract::AccountWithoutValidations(CairoVersion::Cairo1),
        FeatureContract::Empty(CairoVersion::Cairo1),
        FeatureContract::FaultyAccount(CairoVersion::Cairo1),
        FeatureContract::TestContract(CairoVersion::Cairo1),
    ];

    for c in variants {
        if c.get_class_hash() == class_hash {
            let result: Result<CasmContractClass, serde_json::Error> = serde_json::from_str(c.get_raw_class().as_str());
            return result.unwrap();
        }
    }
    panic!("No class found for hash: {:?}", class_hash);
}

fn override_class_hash(contract: &FeatureContract) -> StarkHash {
    match contract {
        // FeatureContract::AccountWithLongValidate(_) => ACCOUNT_LONG_VALIDATE_BASE,
        FeatureContract::AccountWithoutValidations(CairoVersion::Cairo0) => {
            let fe = FieldElement::from_dec_str(
                "3043522133089536593636086481152606703984151542874851197328605892177919922063",
            )
            .unwrap();
            StarkHash::from(fe)
        }
        // FeatureContract::Empty(_) => EMPTY_CONTRACT_BASE,
        FeatureContract::ERC20 => {
            let fe = FieldElement::from_dec_str(
                "2553874082637258309275750418379019378586603706497644242041372159420778949015",
            )
            .unwrap();
            StarkHash::from(fe)
        }
        // FeatureContract::FaultyAccount(_) => FAULTY_ACCOUNT_BASE,
        // FeatureContract::LegacyTestContract => LEGACY_CONTRACT_BASE,
        // FeatureContract::SecurityTests => SECURITY_TEST_CONTRACT_BASE,
        FeatureContract::TestContract(CairoVersion::Cairo0) => {
            let fe = FieldElement::from_dec_str(
                "2847229557799212240700619257444410593768590640938595411219122975663286400357",
            )
            .unwrap();
            StarkHash::from(fe)
        }

        _ => contract.get_class_hash().0,
    }
}

pub fn test_state(
    block_context: &BlockContext,
    initial_balances: u128,
    contract_instances: &[(FeatureContract, u8)],
) -> CachedState<SharedState<DictStorage, PedersenHash>> {
    let mut class_hash_to_class = HashMap::new();
    let mut address_to_class_hash = HashMap::new();
    let class_hash_to_compiled_class_hash: HashMap<ClassHash, CompiledClassHash> = HashMap::new();

    // Declare and deploy account and ERC20 contracts.
    let erc20 = FeatureContract::ERC20;
    let erc20_class_hash: ClassHash = ClassHash(override_class_hash(&erc20));
    class_hash_to_class.insert(erc20_class_hash, erc20.get_class());
    address_to_class_hash.insert(block_context.fee_token_address(&FeeType::Eth), erc20_class_hash);
    address_to_class_hash.insert(block_context.fee_token_address(&FeeType::Strk), erc20_class_hash);

    // Set up the rest of the requested contracts.
    for (contract, n_instances) in contract_instances.iter() {
        let class_hash = ClassHash(override_class_hash(contract));
        // assert!(!class_hash_to_class.contains_key(&class_hash));
        class_hash_to_class.insert(class_hash, contract.get_class());
        for instance in 0..*n_instances {
            let instance_address = contract.get_instance_address(instance);
            address_to_class_hash.insert(instance_address, class_hash);
        }
    }

    // Steps to create the initial state:
    // 1. Use the Blockifier primitives to create the initial contracts, fund accounts, etc. This avoids
    //    recomputing the MPT roots for each modification, we can batch updates when creating the
    //    `SharedState`. This also allows us to reuse some Blockifier test functions, ex:
    //    `fund_account()`.
    // 2. Create the initial `SharedState` object. This computes all the MPT roots.
    // 3. Wrap this new shared state inside a Blockifier `CachedState` to prepare for further updates.

    let mut initial_blockifier_state = CachedState::from(DictStateReader {
        address_to_class_hash,
        class_hash_to_class,
        class_hash_to_compiled_class_hash,
        ..Default::default()
    });

    // fund the accounts.
    for (contract, n_instances) in contract_instances.iter() {
        for instance in 0..*n_instances {
            let instance_address = contract.get_instance_address(instance);
            match contract {
                FeatureContract::AccountWithLongValidate(_)
                | FeatureContract::AccountWithoutValidations(_)
                | FeatureContract::FaultyAccount(_) => {
                    fund_account(block_context, instance_address, initial_balances, &mut initial_blockifier_state);
                }
                _ => (),
            }
        }
    }

    // Insert block-related storage data
    let upper_bound_block_number = block_context.block_number.0 - STORED_BLOCK_HASH_BUFFER;
    let block_number = StorageKey::from(upper_bound_block_number);
    let block_hash = stark_felt!(66_u64);

    let block_hash_contract_address = ContractAddress::try_from(stark_felt!(BLOCK_HASH_CONTRACT_ADDRESS)).unwrap();

    initial_blockifier_state.set_storage_at(block_hash_contract_address, block_number, block_hash).unwrap();

    // TODO:
    let block_info = Default::default();
    let ffc = FactFetchingContext::<_, PedersenHash>::new(Default::default());
    let default_general_config = StarknetGeneralConfig::default(); // TODO
    let shared_state = execute_coroutine_threadsafe(async {
        let shared_state = SharedState::from_blockifier_state(
            ffc,
            initial_blockifier_state.state,
            block_info,
            &default_general_config,
        )
        .await
        .expect("failed to apply initial state as updates to SharedState");

        shared_state
    });

    let cached_state = CachedState::from(shared_state);

    cached_state
}

pub fn os_hints(
    block_context: &BlockContext,
    mut blockifier_state: CachedState<SharedState<DictStorage, PedersenHash>>,
    transactions: Vec<InternalTransaction>,
    tx_execution_infos: Vec<TransactionExecutionInfo>,
) -> (StarknetOsInput, ExecutionHelperWrapper) {
    let shared_state = &blockifier_state.state;
    let mut contracts: HashMap<Felt252, ContractState> = shared_state
        .contract_addresses()
        .iter()
        .map(|address_biguint| {
            // TODO: biguint is exacerbating the type conversion problem, ideas...?
            let address: ContractAddress =
                ContractAddress(PatriciaKey::try_from(felt_vm2api(Felt252::from(address_biguint))).unwrap());
            let contract_state =
                execute_coroutine_threadsafe(async { shared_state.get_contract_state(address) }).unwrap();

            (to_felt252(address.0.key()), contract_state)
        })
        .collect();

    let mut deprecated_compiled_classes: HashMap<Felt252, DeprecatedContractClass> = Default::default();
    let mut compiled_classes: HashMap<Felt252, CasmContractClass> = Default::default();
    let mut class_hash_to_compiled_class_hash: HashMap<Felt252, Felt252> = Default::default();

    for c in contracts.keys() {
        let class_hash = blockifier_state
            .get_class_hash_at(
                ContractAddress::try_from(StarkHash::try_from(c.to_hex_string().as_str()).unwrap()).unwrap(),
            )
            .unwrap();
        let blockifier_class = blockifier_state.get_compiled_contract_class(class_hash).unwrap();
        match blockifier_class {
            V0(_) => {
                deprecated_compiled_classes.insert(to_felt252(&class_hash.0), deprecated_compiled_class(class_hash));
            }
            V1(_) => {
                let class = compiled_class(class_hash);
                let compiled_class_hash = class.compiled_class_hash();
                compiled_classes.insert(Felt252::from_bytes_be(&class.compiled_class_hash().to_be_bytes()), class);
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
        println!("\t{} -> {}", a, BigUint::from_bytes_be(&c.contract_hash));
    }

    println!("deprecated classes");
    for (c, _) in &deprecated_compiled_classes {
        println!("\t{}", c);
    }

    println!("classes");
    for (c, _) in &compiled_classes {
        println!("\t{}", c);
    }

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
    let contract_storage_map =
        build_starknet_storage(blockifier_state).expect("Building the storage map should not fail");

    let execution_helper = ExecutionHelperWrapper::new(
        contract_storage_map,
        tx_execution_infos,
        &block_context,
        (Felt252::from(block_context.block_number.0 - STORED_BLOCK_HASH_BUFFER), Felt252::from(66_u64)),
    );

    (os_input, execution_helper)
}
