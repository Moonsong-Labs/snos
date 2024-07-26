use blockifier::abi::abi_utils::selector_from_name;
use blockifier::context::BlockContext;
use blockifier::invoke_tx_args;
use blockifier::test_utils::NonceManager;
use blockifier::transaction::test_utils;
use blockifier::transaction::test_utils::max_fee;
use rstest::rstest;
use starknet_api::hash::StarkFelt;
use starknet_api::stark_felt;
use starknet_api::transaction::{Calldata, Fee, TransactionVersion};

use crate::common::block_context;
use crate::common::state::{initial_state_cairo1, StarknetTestState};
use crate::common::transaction_utils::execute_txs_and_run_os;

#[rstest]
// We need to use the multi_thread runtime to use task::block_in_place for sync -> async calls.
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn invoke_txn_multiple_calls(
    #[future] initial_state_cairo1: StarknetTestState,
    block_context: BlockContext,
    max_fee: Fee,
) {
    let initial_state = initial_state_cairo1.await;

    let mut nonce_manager = NonceManager::default();

    let sender_address = initial_state.deployed_cairo1_contracts.get("account_with_dummy_validate").unwrap().address;
    let contract_address = initial_state.deployed_cairo0_contracts.get("test_contract").unwrap().address;

    let entrypoint_selector = selector_from_name("return_result").0;
    let calldata = Calldata(
        vec![
            stark_felt!(2u64),
            *contract_address.key(),
            entrypoint_selector,
            stark_felt!(1u64),
            stark_felt!(42u64),
            *contract_address.key(),
            entrypoint_selector,
            stark_felt!(1u64),
            stark_felt!(300u64),
        ]
        .into(),
    );

    let return_result_tx = test_utils::account_invoke_tx(invoke_tx_args! {
        max_fee,
        sender_address,
        calldata,
        version: TransactionVersion::THREE,
        nonce: nonce_manager.next(sender_address),
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
