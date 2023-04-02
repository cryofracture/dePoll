#[cfg(test)]
mod tests {
    use alloc::{
        string::{String, ToString},
        vec::{self, Vec},
    };

    use casper_contract::{
        contract_api::{runtime, storage},
        unwrap_or_revert::UnwrapOrRevert,
    };
    use casper_types::{
        api_error::ApiError,
        contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, NamedKeys},
        CLType, Key, Parameter,
    };

    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
        DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_GENESIS_CONFIG,
        DEFAULT_GENESIS_CONFIG_HASH, DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
    };
    use casper_execution_engine::core::engine_state::{
        run_genesis_request::RunGenesisRequest, GenesisAccount,
    };
    use casper_types::{
        account::AccountHash, runtime_args, Motes, PublicKey, RuntimeArgs, SecretKey, U512,
    };


    const CONTRACT_QUESTION_KEY: &str = "dePoll_question";
    const CONTRACT_OPTIONS_KEY: &str = "dePoll_options";
    const CONTRACT_VOTES_KEY: &str = "dePoll_votes";
    const CONTRACT_HASH: &str = "depoll_contract_hash";
    const CONTRACT_PACKAGE_HASH: &str = "depoll_contract_package_hash";
    const CONTRACT_VERSION_KEY: &str = "version";
    const RUNTIME_QUESTION_ARG: &str = "question";
    const RUNTIME_OPTIONS_ARG: &str = "option";
    const ENTRY_POINT_VOTE: &str = "vote";
    const RUNTIME_VOTE_ARG: &str = "vote_for";
    const ENTRY_POINT_INIT: &str = "init";

    #[test]
    fn should_ask_red_yellow_blue() {
        // Create keypair.
        let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);

        // Create an AccountHash from a public key.
        let account_addr = AccountHash::from(&public_key);
        // Create a GenesisAccount.
        let account = GenesisAccount::account(
            public_key,
            Motes::new(U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE)),
            None,
        );

        let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
        genesis_config.ee_config_mut().push_account(account);

        let run_genesis_request = RunGenesisRequest::new(
            *DEFAULT_GENESIS_CONFIG_HASH,
            genesis_config.protocol_version(),
            genesis_config.take_ee_config(),
        );
        // The test framework checks for compiled Wasm files in '<current working dir>/wasm'.  Paths
        // relative to the current working dir (e.g. 'wasm/contract.wasm') can also be used, as can
        // absolute paths.
        let session_code = PathBuf::from(CONTRACT_WASM);
        let session_args = runtime_args! {
            RUNTIME_QUESTION_ARG => "Favorite color?",
            RUNTIME_OPTIONS_ARG => "",
        };

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {
                ARG_AMOUNT => *DEFAULT_PAYMENT
            })
            .with_session_code(session_code, session_args)
            .with_authorization_keys(&[account_addr])
            .with_address(account_addr)
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();

        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&run_genesis_request).commit();

        // prepare assertions.
        let result_of_query = builder.query(
            None,
            Key::Account(*DEFAULT_ACCOUNT_ADDR),
            &[KEY.to_string()],
        );
        assert!(result_of_query.is_err());

        // deploy the contract.
        builder.exec(execute_request).commit().expect_success();

        // make assertions
        let result_of_query = builder
            .query(None, Key::Account(account_addr), &[KEY.to_string()])
            .expect("should be stored value.")
            .as_cl_value()
            .expect("should be cl value.")
            .clone()
            .into_t::<String>()
            .expect("should be string.");

        assert_eq!(result_of_query, VALUE);
    }

    #[test]
    fn should_error_on_missing_runtime_arg() {
        let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);
        let account_addr = AccountHash::from(&public_key);

        let session_code = PathBuf::from(CONTRACT_WASM);
        let session_args = RuntimeArgs::new();

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_authorization_keys(&[account_addr])
            .with_address(*DEFAULT_ACCOUNT_ADDR)
            .with_session_code(session_code, session_args)
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();

        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();
        builder.exec(execute_request).commit().expect_failure();
    }
}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}
