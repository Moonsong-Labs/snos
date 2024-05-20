use std::collections::{HashMap, HashSet};

use blockifier::state::cached_state::CommitmentStateDiff;
use cairo_vm::types::errors::math_errors::MathError;
use cairo_vm::Felt252;

use crate::config::{
    StarknetGeneralConfig, COMPILED_CLASS_HASH_COMMITMENT_TREE_HEIGHT, CONTRACT_ADDRESS_BITS,
    CONTRACT_STATES_COMMITMENT_TREE_HEIGHT, GLOBAL_STATE_VERSION,
};
use crate::crypto::poseidon::poseidon_hash_many_bytes;
use crate::starknet::business_logic::fact_state::contract_class_objects::{
    get_ffc_for_contract_class_facts, ContractClassLeaf,
};
use crate::starknet::business_logic::fact_state::contract_state_objects::ContractState;
use crate::starknet::business_logic::state::state_api_objects::BlockInfo;
use crate::starkware_utils::commitment_tree::base_types::{Height, TreeIndex};
use crate::starkware_utils::commitment_tree::binary_fact_tree::BinaryFactTree;
use crate::starkware_utils::commitment_tree::errors::TreeError;
use crate::starkware_utils::commitment_tree::patricia_tree::patricia_tree::PatriciaTree;
use crate::storage::storage::{FactFetchingContext, HashFunctionType, Storage};
use crate::utils::felt_api2vm;

/// A class representing a combination of the onchain and offchain state.
pub struct SharedState {
    contract_states: PatriciaTree,
    /// Leaf addresses are class hashes; leaf values contain compiled class hashes.
    contract_classes: Option<PatriciaTree>,
    block_info: BlockInfo,
}

impl SharedState {
    pub fn state_version() -> Felt252 {
        Felt252::from_bytes_be_slice(GLOBAL_STATE_VERSION)
    }

    /// Returns an empty contract state tree.
    pub async fn create_empty_contract_states<S, H>(
        ffc: &mut FactFetchingContext<S, H>,
    ) -> Result<PatriciaTree, TreeError>
    where
        S: Storage + Send + Sync + 'static,
        H: HashFunctionType + Send + Sync + 'static,
    {
        let empty_contract_state =
            ContractState::empty(Height(CONTRACT_STATES_COMMITMENT_TREE_HEIGHT as u64), ffc).await?;
        PatriciaTree::empty_tree(ffc, Height(CONTRACT_ADDRESS_BITS as u64), empty_contract_state).await
    }

    /// Returns an empty contract class tree.
    async fn create_empty_contract_class_tree<S, H>(
        ffc: &mut FactFetchingContext<S, H>,
    ) -> Result<PatriciaTree, TreeError>
    where
        S: Storage + Send + Sync + 'static,
        H: HashFunctionType + Send + Sync + 'static,
    {
        PatriciaTree::empty_tree(
            ffc,
            Height(COMPILED_CLASS_HASH_COMMITMENT_TREE_HEIGHT as u64),
            ContractClassLeaf::empty(),
        )
        .await
    }

    /// Returns an empty state. This is called before creating very first block.
    pub async fn empty<S, H>(
        ffc: &mut FactFetchingContext<S, H>,
        config: &StarknetGeneralConfig,
    ) -> Result<Self, TreeError>
    where
        S: Storage + Send + Sync + 'static,
        H: HashFunctionType + Send + Sync + 'static,
    {
        let empty_contract_states = Self::create_empty_contract_states(ffc).await?;
        let empty_contract_classes = Self::create_empty_contract_class_tree(ffc).await?;

        Ok(Self {
            contract_states: empty_contract_states,
            contract_classes: Some(empty_contract_classes),
            block_info: BlockInfo::empty(Some(felt_api2vm(*config.sequencer_address.0.key())), config.use_kzg_da),
        })
    }

    /// Returns the state's contract class Patricia tree if it exists;
    /// Otherwise returns an empty tree.
    pub async fn get_contract_class_tree<S, H>(
        &self,
        ffc: &mut FactFetchingContext<S, H>,
    ) -> Result<PatriciaTree, TreeError>
    where
        S: Storage + Send + Sync + 'static,
        H: HashFunctionType + Send + Sync + 'static,
    {
        match &self.contract_classes {
            Some(tree) => Ok(tree.clone()),
            None => Self::create_empty_contract_class_tree(ffc).await,
        }
    }
    /// Returns the global state root.
    /// If both the contract class and contract state trees are empty, the global root is set to
    /// 0. If no contract class state exists or if it is empty, the global state root is equal to
    /// the contract state root (for backward compatibility);
    /// Otherwise, the global root is obtained by:
    /// global_root =  H(state_version, contract_state_root, contract_class_root).
    fn get_global_state_root(&self) -> Result<Felt252, MathError> {
        let contract_states_root = &self.contract_states.root;

        let empty_tree_root = vec![0u8; 32];
        let contract_classes_root = match &self.contract_classes {
            Some(tree) => &tree.root,
            None => &empty_tree_root,
        };

        if *contract_states_root == empty_tree_root && *contract_classes_root == empty_tree_root {
            // The shared state is empty.
            return Ok(Felt252::ZERO);
        }

        // Backward compatibility; Used during the migration from a state without a
        // contract class tree to a state with a contract class tree.
        if *contract_classes_root == empty_tree_root {
            // The contract classes' state is empty.
            return Ok(Felt252::from_bytes_be_slice(contract_states_root));
        }

        // Return H(contract_state_root, contract_class_root, state_version).
        poseidon_hash_many_bytes(&[&Self::state_version().to_bytes_be(), contract_states_root, contract_classes_root])
            .map(|x| Felt252::from_bytes_be_slice(&x))
    }

    /// Updates the global state using a state diff generated with Blockifier.
    async fn apply_commitment_state_diff<S, H>(
        self,
        ffc: &mut FactFetchingContext<S, H>,
        state_diff: CommitmentStateDiff,
        block_info: BlockInfo,
    ) -> Result<Self, TreeError>
    where
        S: Storage + 'static,
        H: HashFunctionType + Send + Sync + 'static,
    {
        // We just need to translate the Blockifier state diff to use Felt252 instead of
        // the Blockifier types
        let address_to_class_hash: HashMap<_, _> = state_diff
            .address_to_class_hash
            .into_iter()
            .map(|(address, class_hash)| (felt_api2vm(*address.0.key()), felt_api2vm(class_hash.0)))
            .collect();

        let address_to_nonce: HashMap<_, _> = state_diff
            .address_to_nonce
            .into_iter()
            .map(|(address, nonce)| (felt_api2vm(*address.0.key()), felt_api2vm(nonce.0)))
            .collect();

        let class_hash_to_compiled_class_hash: HashMap<_, _> = state_diff
            .class_hash_to_compiled_class_hash
            .into_iter()
            .map(|(class_hash, compiled_class_hash)| (felt_api2vm(class_hash.0), felt_api2vm(compiled_class_hash.0)))
            .collect();

        let storage_updates: HashMap<_, HashMap<_, _>> = state_diff
            .storage_updates
            .into_iter()
            .map(|(address, contract_storage_updates)| {
                (
                    felt_api2vm(*address.0.key()),
                    contract_storage_updates
                        .into_iter()
                        .map(|(k, v)| (felt_api2vm(*k.0.key()), felt_api2vm(v)))
                        .collect(),
                )
            })
            .collect();

        self.apply_state_updates(
            ffc,
            address_to_class_hash,
            address_to_nonce,
            class_hash_to_compiled_class_hash,
            storage_updates,
            block_info,
        )
        .await
    }

    async fn apply_state_updates<S, H>(
        mut self,
        ffc: &mut FactFetchingContext<S, H>,
        address_to_class_hash: HashMap<Felt252, Felt252>,
        address_to_nonce: HashMap<Felt252, Felt252>,
        class_hash_to_compiled_class_hash: HashMap<Felt252, Felt252>,
        storage_updates: HashMap<Felt252, HashMap<Felt252, Felt252>>,
        block_info: BlockInfo,
    ) -> Result<Self, TreeError>
    where
        S: Storage + 'static,
        H: HashFunctionType + Send + Sync + 'static,
    {
        let accessed_addresses_felts: HashSet<_> =
            address_to_class_hash.keys().chain(address_to_nonce.keys().chain(storage_updates.keys())).collect();
        let accessed_addresses: Vec<TreeIndex> = accessed_addresses_felts.iter().map(|x| x.to_biguint()).collect();

        let mut facts = None;
        let mut current_contract_states: HashMap<TreeIndex, ContractState> =
            self.contract_states.get_leaves(ffc, &accessed_addresses, &mut facts).await?;

        // Update contract storage roots with cached changes.
        let empty_updates = HashMap::new();
        let mut updated_contract_states = HashMap::new();
        for address in accessed_addresses_felts {
            // unwrap() is safe as an entry is guaranteed to be present with `get_leaves()`.
            let tree_index = address.to_biguint();
            let updates = storage_updates.get(&address).unwrap_or(&empty_updates);
            let nonce = address_to_nonce.get(&address).cloned();
            let class_hash = address_to_class_hash.get(&address).cloned();
            let updated_contract_state =
                current_contract_states.remove(&tree_index).unwrap().update(ffc, updates, nonce, class_hash).await?;

            updated_contract_states.insert(tree_index, updated_contract_state);
        }

        // Apply contract changes on global root.
        println!("Updating contract state tree with {} modifications...", accessed_addresses.len());
        let global_state_modifications: Vec<_> = updated_contract_states.into_iter().map(|(k, v)| (k, v)).collect();
        let updated_global_contract_root =
            self.contract_states.update(ffc, global_state_modifications, &mut facts).await?;

        let mut ffc_for_contract_class = get_ffc_for_contract_class_facts(ffc);

        let updated_contract_classes = match self.contract_classes {
            Some(mut tree) => {
                println!(
                    "Updating contract class tree with {} modifications...",
                    class_hash_to_compiled_class_hash.len()
                );
                let modifications: Vec<_> = class_hash_to_compiled_class_hash
                    .into_iter()
                    .map(|(key, value)| (key.to_biguint(), ContractClassLeaf::create(value)))
                    .collect();
                Some(tree.update(&mut ffc_for_contract_class, modifications, &mut facts).await?)
            }
            None => {
                assert_eq!(
                    class_hash_to_compiled_class_hash.len(),
                    0,
                    "contract_classes must be concrete before update."
                );
                None
            }
        };

        Ok(Self {
            contract_states: updated_global_contract_root,
            contract_classes: updated_contract_classes,
            block_info,
        })
    }
}

//     def get_global_state_root(self) -> int:
//         """
//         Returns the global state root.
//         If both the contract class and contract state trees are empty, the global root is set to
// 0. If no contract class state exists or if it is empty, the global state root is equal to
// the         contract state root (for backward compatibility);
//         Otherwise, the global root is obtained by:
//             global_root =  H(state_version, contract_state_root, contract_class_root).
//         """
//         contract_states_root = self.contract_states.root
//         contract_classes_root = (
//             self.contract_classes.root if self.contract_classes is not None else to_bytes(0)
//         )
//
//         if contract_states_root == to_bytes(0) and contract_classes_root == to_bytes(0):
//             # The shared state is empty.
//             return 0
//
//         # Backward compatibility; Used during the migration from a state without a
//         # contract class tree to a state with a contract class tree.
//         if contract_classes_root == to_bytes(0):
//             # The contract classes' state is empty.
//             return from_bytes(contract_states_root)
//
//         # Return H(contract_state_root, contract_class_root, state_version).
//         hash_value = poseidon_hash_many(
//             [
//                 self.state_version,
//                 from_bytes(contract_states_root),
//                 from_bytes(contract_classes_root),
//             ]
//         )
//         return hash_value
//
//     def to_carried_state(self, ffc: FactFetchingContext) -> CarriedState:
//         state = CachedState(
//             block_info=self.block_info,
//             state_reader=PatriciaStateReader(
//                 contract_state_root=self.contract_states,
//                 contract_class_root=self.contract_classes,
//                 ffc=ffc,
//                 contract_class_storage=ffc.storage,
//             ),
//         )
//         return CarriedState(parent_state=None, state=state)
//
//     async def get_filled_carried_state(
//         self, ffc: FactFetchingContext, state_selector: StateSelectorBase
//     ) -> CarriedState:
//         raise NotImplementedError(
//             "get_filled_carried_state() is not implemented on Starknet SharedState."
//         )
//
//     async def apply_state_updates(
//         self,
//         ffc: FactFetchingContext,
//         previous_carried_state: CarriedStateBase,
//         current_carried_state: CarriedStateBase,
//         facts: Optional[BinaryFactDict] = None,
//     ) -> "SharedState":
//         # Note that previous_carried_state is part of the API of
//         # SharedStateBase.apply_state_updates().
//
//         # Downcast arguments to application-specific types.
//         assert isinstance(previous_carried_state, CarriedState)
//         assert isinstance(current_carried_state, CarriedState)
//
//         state_objects_logger.debug(
//             f"Updating state from previous carried state: {previous_carried_state} "
//             f"to current carried state: {current_carried_state}"
//         )
//
//         # Prepare storage updates to apply.
//         state_cache = current_carried_state.state.cache
//         return await self.apply_updates(
//             ffc=ffc,
//             address_to_class_hash=state_cache._class_hash_writes,
//             address_to_nonce=state_cache._nonce_writes,
//             class_hash_to_compiled_class_hash=state_cache._compiled_class_hash_writes,
//             storage_updates=to_state_diff_storage_mapping(
//                 storage_writes=state_cache._storage_writes
//             ),
//             block_info=current_carried_state.state.block_info,
//         )
//
//     async def apply_updates(
//         self,
//         ffc: FactFetchingContext,
//         address_to_class_hash: Mapping[int, int],
//         address_to_nonce: Mapping[int, int],
//         class_hash_to_compiled_class_hash: Mapping[int, int],
//         storage_updates: Mapping[int, Mapping[int, int]],
//         block_info: BlockInfo,
//     ) -> "SharedState":
//         accessed_addresses = (
//             address_to_class_hash.keys() | address_to_nonce.keys() | storage_updates.keys()
//         )
//         current_contract_states = await self.contract_states.get_leaves(
//             ffc=ffc, indices=accessed_addresses, fact_cls=ContractState
//         )
//
//         # Update contract storage roots with cached changes.
//         updated_contract_states = await gather_in_chunks(
//             awaitables=(
//                 current_contract_states[address].update(
//                     ffc=ffc,
//                     updates=storage_updates.get(address, {}),
//                     nonce=address_to_nonce.get(address, None),
//                     class_hash=address_to_class_hash.get(address, None),
//                 )
//                 for address in accessed_addresses
//             )
//         )
//
//         # Apply contract changes on global root.
//         logger.info(f"Updating contract state tree with {len(accessed_addresses)}
// modifications...")         updated_global_contract_root = await
// self.contract_states.update_efficiently(             ffc=ffc,
// modifications=list(safe_zip(accessed_addresses, updated_contract_states))         )
//
//         ffc_for_contract_class = get_ffc_for_contract_class_facts(ffc=ffc)
//         updated_contract_classes: Optional[PatriciaTree] = None
//         if self.contract_classes is not None:
//             logger.info(
//                 f"Updating contract class tree with {len(class_hash_to_compiled_class_hash)} "
//                 "modifications..."
//             )
//             updated_contract_classes = await self.contract_classes.update_efficiently(
//                 ffc=ffc_for_contract_class,
//                 modifications=[
//                     (key, ContractClassLeaf.create(compiled_class_hash=value))
//                     for key, value in class_hash_to_compiled_class_hash.items()
//                 ],
//             )
//         else:
//             assert (
//                 len(class_hash_to_compiled_class_hash) == 0
//             ), "contract_classes must be concrete before update."
//
//         return SharedState(
//             contract_states=updated_global_contract_root,
//             contract_classes=updated_contract_classes,
//             block_info=block_info,
//         )
