// An interface for setting and getting a single value
#[starknet::interface]
trait GetSetSingleValue<TContractState> {
    fn set_value(ref self: TContractState, _value: felt252);
    fn get_value(self: @TContractState) -> felt252;
}

// A contract which implements the interface but also uses the interface to
// call into itself recursively
#[starknet::contract]
mod TestRecursiveStorageContract {
    use box::BoxTrait;
    use dict::Felt252DictTrait;
    use ec::EcPointTrait;
    use starknet::ClassHash;
    use starknet::ContractAddress;
    use starknet::get_execution_info;
    use starknet::StorageAddress;
    use array::ArrayTrait;
    use clone::Clone;
    use core::bytes_31::POW_2_128;
    use core::integer::bitwise;
    use traits::Into;
    use traits::TryInto;
    use starknet::{
        eth_address::U256IntoEthAddress, EthAddress, secp256_trait::{Signature, is_valid_signature},
        secp256r1::{Secp256r1Point, Secp256r1Impl}, eth_signature::verify_eth_signature,
        info::{BlockInfo, SyscallResultTrait}, info::v2::{ExecutionInfo, TxInfo, ResourceBounds,},
        syscalls
    };

    #[storage]
    struct Storage {
        single_value: felt252,
    }

    #[generate_trait]
    // #[external(v0)]
    impl TRSCImpl of TestRecursiveStorageContract {
        fn set_value(ref self: ContractState, value: felt252) {
            self.single_value.write(value);
        }
        fn get_value(self: @ContractState) -> felt252 {
            self.single_value.read()
        }
    }

    #[external(v0)]
    fn set_value_direct(ref self: ContractState, value: felt252) {
        self.single_value.write(value);
    }

    #[external(v0)]
    fn get_value_direct(self: @ContractState) -> felt252 {
        self.single_value.read()
    }

    #[constructor]
    fn constructor(ref self: ContractState, initial_value: felt252) {
        self.single_value.write(initial_value);
    }

    #[external(v0)]
    fn test_storage_read_write(
        self: @ContractState, address: StorageAddress, value: felt252
    ) -> felt252 {
        let address_domain = 0;
        syscalls::storage_write_syscall(address_domain, address, value).unwrap_syscall();
        syscalls::storage_read_syscall(address_domain, address).unwrap_syscall()
    }

    #[external(v0)]
    fn test_count_actual_storage_changes(self: @ContractState) {
        let storage_address = 15.try_into().unwrap();
        let address_domain = 0;
        syscalls::storage_write_syscall(address_domain, storage_address, 0).unwrap_syscall();
        syscalls::storage_write_syscall(address_domain, storage_address, 1).unwrap_syscall();
    }

    #[external(v0)]
    #[raw_output]
    fn test_call_contract(
        self: @ContractState,
        contract_address: ContractAddress,
        entry_point_selector: felt252,
        calldata: Array::<felt252>
    ) -> Span::<felt252> {
        syscalls::call_contract_syscall(contract_address, entry_point_selector, calldata.span())
            .unwrap_syscall()
            .snapshot
            .span()
    }

    #[external(v0)]
    #[raw_output]
    fn test_nested_library_call(
        self: @ContractState,
        class_hash: ClassHash,
        lib_selector: felt252,
        nested_selector: felt252,
        a: felt252,
        b: felt252
    ) -> Span::<felt252> {
        let mut nested_library_calldata: Array::<felt252> = Default::default();
        nested_library_calldata.append(class_hash.into());
        nested_library_calldata.append(nested_selector);
        nested_library_calldata.append(2);
        nested_library_calldata.append(a + 1);
        nested_library_calldata.append(b + 1);
        let _res = starknet::library_call_syscall(
            class_hash, lib_selector, nested_library_calldata.span(),
        )
            .unwrap_syscall();

        let mut calldata: Array::<felt252> = Default::default();
        calldata.append(a);
        calldata.append(b);
        starknet::library_call_syscall(class_hash, nested_selector, calldata.span())
            .unwrap_syscall()
    }

    #[external(v0)]
    fn test_deploy(
        self: @ContractState,
        class_hash: ClassHash,
        contract_address_salt: felt252,
        calldata: Array::<felt252>,
        deploy_from_zero: bool,
    ) {
        syscalls::deploy_syscall(
            class_hash, contract_address_salt, calldata.span(), deploy_from_zero
        )
            .unwrap_syscall();
    }
}
