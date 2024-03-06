#!/bin/bash

CAIRO_VER="0.13.1a0"

if ! command -v cairo-compile >/dev/null; then
    echo "please start cairo($CAIRO_VER) dev environment"
    exit 1
fi

if ! command -v starknet-compile-deprecated >/dev/null; then
    echo "please start cairo($CAIRO_VER) dev environment"
    exit 1
fi

echo -e "\ninitializing cairo-lang($CAIRO_VER)...\n"
git submodule update --init

FETCHED_CAIRO_VER="$(cat cairo-lang/src/starkware/cairo/lang/VERSION)"

if [ "$CAIRO_VER" != "$FETCHED_CAIRO_VER" ]; then
    echo "incorrect cairo ver($FETCHED_CAIRO_VER) expecting $CAIRO_VER"
    exit 1
fi

# setup test_contract path
echo -e "setting up cairo dependencies...\n"
cp tests/dependencies/test_contract_interface.cairo cairo-lang/src/starkware/starknet/core/test_contract/
cp tests/dependencies/deprecated_syscalls.cairo cairo-lang/src/starkware/starknet/core/test_contract/

# setup token_for_testing path
mkdir -p cairo-lang/src/starkware/starknet/std_contracts/ERC20
cp tests/dependencies/ERC20.cairo cairo-lang/src/starkware/starknet/std_contracts/ERC20/
cp tests/dependencies/ERC20_base.cairo cairo-lang/src/starkware/starknet/std_contracts/ERC20/
cp tests/dependencies/permitted.cairo cairo-lang/src/starkware/starknet/std_contracts/ERC20/
mkdir -p cairo-lang/src/starkware/starknet/std_contracts/upgradability_proxy
cp tests/dependencies/initializable.cairo cairo-lang/src/starkware/starknet/std_contracts/upgradability_proxy

# compile cairo programs
echo -e "compiling cairo programs...\n"
mkdir -p build/programs
cairo-format -i tests/programs/*
cairo-compile tests/programs/bad_output.cairo --output build/programs/bad_output.json
cairo-compile tests/programs/fact.cairo --output build/programs/fact.json

# compile os with debug info
cairo-compile --debug_info_with_source cairo-lang/src/starkware/starknet/core/os/os.cairo --output build/os_debug.json --cairo_path cairo-lang/src
cairo-compile cairo-lang/src/starkware/starknet/core/os/os.cairo --output build/os_latest.json --cairo_path cairo-lang/src

# compile starknet contract
echo -e "compiling starknet contracts...\n"
mkdir -p build/contracts
mkdir -p build/pie
ln -sf cairo-lang/src/starkware starkware
starknet-compile-deprecated tests/contracts/token_for_testing.cairo --output build/contracts/token_for_testing.json --cairo_path cairo-lang/src --account_contract
starknet-compile-deprecated tests/contracts/dummy_account.cairo --output build/contracts/dummy_account.json --cairo_path cairo-lang/src --account_contract
starknet-compile-deprecated tests/contracts/dummy_token.cairo --output build/contracts/dummy_token.json --cairo_path cairo-lang/src --account_contract
starknet-compile-deprecated tests/contracts/delegate_proxy.cairo --output build/contracts/delegate_proxy.json --cairo_path cairo-lang/src
starknet-compile-deprecated tests/contracts/test_contract.cairo --output build/contracts/test_contract.json --cairo_path cairo-lang/src
starknet-compile-deprecated tests/contracts/test_contract2.cairo --output build/contracts/test_contract2.json --cairo_path cairo-lang/src
