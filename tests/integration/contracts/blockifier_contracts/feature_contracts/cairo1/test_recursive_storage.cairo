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
    use starknet::ClassHash;
    use starknet::ContractAddress;
    use starknet::StorageAddress;
    use starknet::{
        info::SyscallResultTrait,
        syscalls
    };
    use super::{GetSetSingleValueDispatcher, GetSetSingleValueDispatcherTrait};

    #[storage]
    struct Storage {
        single_value: felt252,
    }

    #[generate_trait]
    #[abi(per_item)]
    impl GetSetSingleValue of TestRecursiveStorageContract {
        #[external(v0)]
        fn set_value(ref self: ContractState, value: felt252) {
            self.single_value.write(value);
        }
        #[external(v0)]
        fn get_value(self: @ContractState) -> felt252 {
            self.single_value.read()
        }
    }

    #[constructor]
    fn constructor(ref self: ContractState, initial_value: felt252) {
        self.single_value.write(initial_value);
    }

    #[generate_trait]
    impl InternalFunctions of InternalFunctionsTrait {

        #[external(v0)]
        fn set_value_direct(ref self: ContractState, value: felt252) {
            self.single_value.write(value);
        }

        #[external(v0)]
        fn get_value_direct(self: @ContractState) -> felt252 {
            self.single_value.read()
        }

        #[external(v0)]
        fn set_value_indirect(ref self: ContractState, addr: ContractAddress, value: felt252) {
            GetSetSingleValueDispatcher {contract_address: addr}.set_value(value);
        }

        #[external(v0)]
        fn get_value_indirect(ref self: ContractState, addr: ContractAddress) -> felt252 {
            GetSetSingleValueDispatcher {contract_address: addr}.get_value()
        }
    }

    #[external(v0)]
    fn test_storage_replay_system(ref self: ContractState, addr: ContractAddress) {

        // set value directly (within this same call)
        self.set_value_direct(2);

        // set value in a subcall -- this will execute out of order in SNOS
        self.set_value_indirect(addr, 1);

        // set value directly again
        self.set_value_direct(42);

        // TODO: this doesn't do anything for testing out-of-order storage replays since it occurs
        //       within the same call
        // get value directly, should be 42
        let value = self.get_value_direct();
        assert(value == 42, 'INVALID_FINAL_STATE_VALUE');
    }

    #[external(v0)]
    fn expect_value(ref self: ContractState, expected: felt252) {
        let value = self.get_value_direct();
        assert(value == expected, 'UNEXPECTED_VALUE');
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
