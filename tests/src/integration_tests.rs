#[cfg(test)]
mod tests {
    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder,
        ARG_AMOUNT, DEFAULT_ACCOUNT_ADDR, DEFAULT_PAYMENT, PRODUCTION_RUN_GENESIS_REQUEST
    };
    use casper_contract::{
        contract_api::{runtime, storage},
    };
    use casper_execution_engine::core::{engine_state::Error as EngineStateError, execution};
    use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
    use casper_types::ContractHash;
    use casper_types::{api_error::ApiError, Key};
    use casper_types::{runtime_args, RuntimeArgs};
    use std::path::PathBuf;

    // NamedKey and DictKey Values
    const CONTRACT_QUESTION_KEY: &str = "dePoll_question";
    const CONTRACT_KEY_OPTIONS: &str = "dePoll_options";
    const CONTRACT_KEY_OPTIONS_COUNT: &str = "dePoll_option_count";
    const CONTRACT_OPTIONS_DICT_UREF: &str = "dePoll_dict_seed_uref";
    const CONTRACT_ACCESS_KEY: &str = "dePoll_contract_access_key";
    const CONTRACT_HASH: &str = "dePoll_contract_hash";
    const CONTRACT_PACKAGE: &str = "dePoll_contract_package";
    const CONTRACT_KEY_POLL_START: &str = "poll_start";
    const CONTRACT_KEY_POLL_END: &str = "poll_end";
    const CONTRACT_VERSION_KEY: &str = "dePoll_version";
    const CONTRACT_KEY_OPTION_ONE: &str = "dePoll_option_one";
    const CONTRACT_KEY_OPTION_TWO: &str = "dePoll_option_two";
    const CONTRACT_KEY_OPTION_X: &str = "dePoll_option_";

    // Contract Constants
    const INSTALLER: &str = "installer";
    const INITIAL_VOTE_COUNT: u64 = 0;
    const SECONDS_PER_MIN: u64 = 60;
    const MILLI_PER_SEC: u64 = 1000;
    const INIT: &str = "init";
    const RUNTIME_VOTE_ARG: &str = "vote_for";
    const KEY_POLL_END: &str = "poll_end";
    const CONTRACT_WASM: &str = "contract.wasm";
    const QUESTION_VALUE: &str = "Favorite color?";
    const RED: &str = "red";
    const GREEN: &str = "green";
    const YELLOW: &str = "yellow";

    // Runtime Arguments
    const RUNTIME_ARG_QUESTION: &str = "question";
    const RUNTIME_ARG_OPTION_ONE: &str = "option_one";
    const RUNTIME_ARG_OPTION_TWO: &str = "option_two";
    const RUNTIME_ARG_ADD_OPTION: &str = "add_poll_option";
    const RUNTIME_ARG_CAST_VOTE: &str = "vote_for";
    const RUNTIME_ARG_POLL_LENGTH: &str = "poll_length";
    const RUNTIME_ARG_EXTEND_POLL: &str = "extend_duration";


    // Entrypoints
    const ENTRY_POINT_VOTE: &str = "vote";
    const ENTRY_POINT_ADD_OPTION: &str = "add_option";
    const ENTRY_POINT_INIT: &str = "init";
    const ENTRY_POINT_EXTEND_POLL: &str = "extend_poll";


    #[test]
    fn should_have_a_stored_question_in_contract_context() {
        let builder = install_contract();

        let contract_hash_ref = runtime::get_key(CONTRACT_HASH).into_uref();
        let contract_hash: ContractHash = storage::read(contract_hash_ref);

        let contract = builder
            .get_contract(contract_hash)
            .expect("should have contract");
        let named_keys = contract.named_keys();
        dbg!(named_keys);
        // make assertion
        let question = builder
            .query(
                None,
                Key::Account(*DEFAULT_ACCOUNT_ADDR),
                &[CONTRACT_HASH.to_string(), CONTRACT_QUESTION_KEY.to_string()],
            )
            .expect("should be stored value.")
            .as_cl_value()
            .expect("should be cl value.")
            .clone()
            .into_t::<String>()
            .expect("should be string.");
        assert_eq!(question, QUESTION_VALUE);
    }

    #[test]
    fn should_have_a_dict_seed_uref_in_installer_context() {
        let builder = install_contract();
        // make assertion
        let dict_seed_uref = builder
            .get_expected_account(*DEFAULT_ACCOUNT_ADDR)
            .named_keys()
            .get(CONTRACT_OPTIONS_DICT_UREF)
            .expect("must have this entry in named keys")
            .into_uref();
        assert!(dict_seed_uref.is_some());
        dbg!(dict_seed_uref);
    }

    #[test]
    fn should_vote_for_red() {
        let mut builder = install_contract();

        let contract_hash = builder
            .get_expected_account(*DEFAULT_ACCOUNT_ADDR)
            .named_keys()
            .get(CONTRACT_HASH)
            .expect("must have this entry in named keys")
            .into_hash()
            .map(ContractHash::new)
            .unwrap();

        let dict_seed_uref = *builder
            .query(None, contract_hash.into(), &[])
            .expect("must have contract hash")
            .as_contract()
            .expect("must convert as contract")
            .named_keys()
            .get(CONTRACT_KEY_OPTIONS)
            .expect("must have key")
            .as_uref()
            .expect("must convert to seed uref");

        let nb_or_red_votes = builder
            .query_dictionary_item(None, dict_seed_uref, RED)
            .expect("should be stored value.")
            .as_cl_value()
            .expect("should be cl value.")
            .clone()
            .into_t::<u64>()
            .expect("should be u64");

        assert_eq!(nb_or_red_votes, INITIAL_VOTE_COUNT);

        let entry_point_deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_authorization_keys(&[*DEFAULT_ACCOUNT_ADDR])
            .with_address(*DEFAULT_ACCOUNT_ADDR)
            .with_stored_session_hash(
                contract_hash,
                ENTRY_POINT_VOTE,
                runtime_args! {
                    RUNTIME_VOTE_ARG => RED
                },
            )
            .build();

        let entry_point_request =
            ExecuteRequestBuilder::from_deploy_item(entry_point_deploy_item).build();

        builder.exec(entry_point_request).expect_success().commit();

        let nb_or_red_votes = builder
            .query_dictionary_item(None, dict_seed_uref, RED)
            .expect("should be stored value.")
            .as_cl_value()
            .expect("should be cl value.")
            .clone()
            .into_t::<u64>()
            .expect("should be u64");

        assert_eq!(nb_or_red_votes, INITIAL_VOTE_COUNT + 1);
    }

    #[test]
    fn should_add_option_green() {
        let mut builder = install_contract();
        let contract_hash = builder
            .get_expected_account(*DEFAULT_ACCOUNT_ADDR)
            .named_keys()
            .get(CONTRACT_HASH)
            .expect("must have this entry in named keys")
            .into_hash()
            .map(ContractHash::new)
            .unwrap();

        let entry_point_deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_authorization_keys(&[*DEFAULT_ACCOUNT_ADDR])
            .with_address(*DEFAULT_ACCOUNT_ADDR)
            .with_stored_session_hash(
                contract_hash,
                ENTRY_POINT_ADD_OPTION,
                runtime_args! {
                    RUNTIME_ARG_ADD_OPTION => GREEN
                },
            )
            .build();

        let entry_point_request =
            ExecuteRequestBuilder::from_deploy_item(entry_point_deploy_item).build();

        builder.exec(entry_point_request).expect_success().commit();

        let entry_point_deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_authorization_keys(&[*DEFAULT_ACCOUNT_ADDR])
            .with_address(*DEFAULT_ACCOUNT_ADDR)
            .with_stored_session_hash(
                contract_hash,
                ENTRY_POINT_VOTE,
                runtime_args! {
                    RUNTIME_ARG_CAST_VOTE => GREEN
                },
            )
            .build();

        let entry_point_request =
            ExecuteRequestBuilder::from_deploy_item(entry_point_deploy_item).build();

        builder.exec(entry_point_request).expect_success().commit();

        let dict_seed_uref = *builder
            .query(None, contract_hash.into(), &[])
            .expect("must have contract hash")
            .as_contract()
            .expect("must convert as contract")
            .named_keys()
            .get(CONTRACT_KEY_OPTIONS)
            .expect("must have key")
            .as_uref()
            .expect("must convert to seed uref");

        let nb_or_green_votes = builder
            .query_dictionary_item(None, dict_seed_uref, GREEN)
            .expect("should be stored value.")
            .as_cl_value()
            .expect("should be cl value.")
            .clone()
            .into_t::<u64>()
            .expect("should be u64");

        assert_eq!(nb_or_green_votes, INITIAL_VOTE_COUNT + 1);
    }

    #[test]
    fn should_error_on_missing_runtime_arg() {
        let session_code = PathBuf::from(CONTRACT_WASM);
        let session_args = RuntimeArgs::new();

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_authorization_keys(&[*DEFAULT_ACCOUNT_ADDR])
            .with_address(*DEFAULT_ACCOUNT_ADDR)
            .with_session_code(session_code, session_args)
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();

        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST).commit();
        builder.exec(execute_request).commit().expect_failure();

        let error = ApiError::MissingArgument;
        let error_code: u32 = error.into();
        let actual_error = builder.get_error().expect("must have error");
        let reason = "should error on missing runtime_arg";
        let actual = format!("{actual_error:?}");
        let expected = format!(
            "{:?}",
            EngineStateError::Exec(execution::Error::Revert(error))
        );

        assert_eq!(
            actual, expected,
            "Error should match {error_code} with reason: {reason}"
        )
    }

    fn install_contract() -> WasmTestBuilder<InMemoryGlobalState> {
        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST).commit();

        let session_code = PathBuf::from(CONTRACT_WASM);
        let session_args = runtime_args! {
            RUNTIME_ARG_QUESTION => QUESTION_VALUE,
            RUNTIME_ARG_OPTION_ONE => RED,
            RUNTIME_ARG_OPTION_TWO => YELLOW,
        };

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {
                ARG_AMOUNT => *DEFAULT_PAYMENT
            })
            .with_session_code(session_code, session_args)
            .with_authorization_keys(&[*DEFAULT_ACCOUNT_ADDR])
            .with_address(*DEFAULT_ACCOUNT_ADDR)
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();

        // prepare assertions.
        let result_of_query = builder.query(
            None,
            Key::Account(*DEFAULT_ACCOUNT_ADDR),
            &[CONTRACT_QUESTION_KEY.to_string()],
        );
        assert!(result_of_query.is_err());

        // deploy the contract.
        builder.exec(execute_request).commit().expect_success();

        let contract_hash = builder
            .query(
                None,
                Key::Account(*DEFAULT_ACCOUNT_ADDR),
                &[CONTRACT_HASH.to_string()],
            )
            .unwrap();
        let installer = contract_hash
            .as_contract()
            .unwrap()
            .named_keys()
            .get(INSTALLER)
            .unwrap();

        assert_eq!(installer, &Key::Account(*DEFAULT_ACCOUNT_ADDR));

        builder
    }
}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}