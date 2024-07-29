use blockifier::abi::abi_utils::selector_from_name;
use blockifier::context::BlockContext;
use blockifier::invoke_tx_args;
use blockifier::test_utils::{create_calldata, NonceManager, BALANCE};
use blockifier::transaction::test_utils;
use blockifier::transaction::test_utils::max_fee;
use rstest::{fixture, rstest};
use starknet_api::hash::StarkFelt;
use starknet_api::stark_felt;
use starknet_api::transaction::{Fee, TransactionVersion};

use crate::common::block_context;
use crate::common::blockifier_contracts::load_cairo0_feature_contract;
use crate::common::openzeppelin::load_upgradable_account;
use crate::common::state::{init_logging, StarknetStateBuilder, StarknetTestState};
use crate::common::transaction_utils::execute_txs_and_run_os;

#[fixture]
async fn initial_state_oz_account(
    block_context: BlockContext,
    #[from(init_logging)] _logging: (),
) -> StarknetTestState {
    let oz_account = load_upgradable_account();
    let test_contract = load_cairo0_feature_contract("test_contract");

    StarknetStateBuilder::new(&block_context)
        .deploy_cairo1_contract(oz_account.0, oz_account.1, oz_account.2)
        .deploy_cairo0_contract(test_contract.0, test_contract.1)
        .set_default_balance(BALANCE, BALANCE)
        .build()
        .await
}

#[rstest]
// We need to use the multi_thread runtime to use task::block_in_place for sync -> async calls.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn invoke_txn_multiple_calls(
    #[future] initial_state_oz_account: StarknetTestState,
    block_context: BlockContext,
    max_fee: Fee,
) {
    let initial_state = initial_state_oz_account.await;

    let mut nonce_manager = NonceManager::default();

    let account_address = initial_state.deployed_cairo1_contracts.get("upgradable_account").unwrap().address;
    let contract_address = initial_state.deployed_cairo0_contracts.get("test_contract").unwrap().address;

    let entrypoint_selector = selector_from_name("return_result").0;
    let return_result_calldata = vec![
        stark_felt!(1u64),
        *contract_address.key(),
        entrypoint_selector,
        stark_felt!(1u64),
        stark_felt!(42u64),
        // *contract_address.key(),
        // entrypoint_selector,
        // stark_felt!(1u64),
        // stark_felt!(300u64),
    ];

    let calldata = create_calldata(account_address, "__execute__", &return_result_calldata);

    let return_result_tx = test_utils::account_invoke_tx(invoke_tx_args! {
        max_fee,
        sender_address: account_address,
        calldata,
        version: TransactionVersion::THREE,
        nonce: nonce_manager.next(account_address),
    });

    let txs = vec![return_result_tx].into_iter().map(Into::into).collect();
    let _result = execute_txs_and_run_os(
        initial_state.cached_state,
        block_context,
        txs,
        initial_state.cairo0_compiled_classes,
        initial_state.cairo1_compiled_classes,
    )
    .await
    .expect("OS run failed");
}
