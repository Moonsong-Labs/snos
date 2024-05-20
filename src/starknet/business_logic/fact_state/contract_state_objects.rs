use cairo_vm::Felt252;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::starknet::starknet_storage::StorageLeaf;
use crate::starkware_utils::commitment_tree::base_types::Height;
use crate::starkware_utils::commitment_tree::binary_fact_tree::BinaryFactTree;
use crate::starkware_utils::commitment_tree::errors::TreeError;
use crate::starkware_utils::commitment_tree::leaf_fact::LeafFact;
use crate::starkware_utils::commitment_tree::patricia_tree::patricia_tree::{PatriciaTree, EMPTY_NODE_HASH};
use crate::starkware_utils::serializable::SerializationPrefix;
use crate::storage::storage::{DbObject, Fact, FactFetchingContext, HashFunctionType, Storage};

pub const UNINITIALIZED_CLASS_HASH: [u8; 32] = [0; 32];

#[serde_as]
#[derive(Deserialize, Clone, Debug, Serialize, PartialEq)]
pub struct ContractState {
    pub contract_hash: Vec<u8>,
    pub storage_commitment_tree: PatriciaTree,
    pub nonce: Felt252,
}

impl SerializationPrefix for ContractState {}

impl ContractState {
    pub fn create(contract_hash: Vec<u8>, storage_commitment_tree: PatriciaTree, nonce: Felt252) -> Self {
        Self { contract_hash, storage_commitment_tree, nonce }
    }

    pub async fn empty<S, H>(
        storage_commitment_tree_height: Height,
        ffc: &mut FactFetchingContext<S, H>,
    ) -> Result<Self, TreeError>
    where
        S: Storage + Send + Sync + 'static,
        H: HashFunctionType + Send + Sync + 'static,
    {
        let empty_tree = PatriciaTree::empty_tree(ffc, storage_commitment_tree_height, StorageLeaf::empty()).await?;
        Ok(Self {
            contract_hash: UNINITIALIZED_CLASS_HASH.to_vec(),
            storage_commitment_tree: empty_tree,
            nonce: Felt252::ZERO,
        })
    }
}

impl<S, H> Fact<S, H> for ContractState
where
    H: HashFunctionType,
    S: Storage,
{
    /// Computes the hash of the node containing the contract's information, including the contract
    /// definition and storage.
    fn hash(&self) -> Vec<u8> {
        if <ContractState as LeafFact<S, H>>::is_empty(self) {
            return EMPTY_NODE_HASH.to_vec();
        }

        let contract_state_hash_version = Felt252::ZERO;
        // Set hash_value = H(H(contract_hash, storage_root), RESERVED).
        let hash_value = H::hash(&self.contract_hash, &self.storage_commitment_tree.root);
        let hash_value = H::hash(&hash_value, &self.nonce.to_bytes_be());

        // Return H(hash_value, CONTRACT_STATE_HASH_VERSION).
        // CONTRACT_STATE_HASH_VERSION must be in the outermost hash to guarantee unique "decoding".
        H::hash(&hash_value, &contract_state_hash_version.to_bytes_be())
    }
}

impl DbObject for ContractState {}

impl<S, H> LeafFact<S, H> for ContractState
where
    S: Storage,
    H: HashFunctionType,
{
    fn is_empty(&self) -> bool {
        self.storage_commitment_tree.root == EMPTY_NODE_HASH
            && self.contract_hash == UNINITIALIZED_CLASS_HASH
            && self.nonce == Felt252::ZERO
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::crypto::pedersen::PedersenHash;
    use crate::storage::dict_storage::DictStorage;

    #[rstest]
    #[tokio::test]
    async fn test_is_empty() {
        let mut ffc = FactFetchingContext::<_, PedersenHash>::new(DictStorage::default());
        let contract_state = ContractState::empty(Height(251), &mut ffc).await.unwrap();

        assert!(<ContractState as LeafFact<DictStorage, PedersenHash>>::is_empty(&contract_state));
    }
}

