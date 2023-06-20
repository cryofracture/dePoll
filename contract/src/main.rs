#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;
use alloc::{
    string::{String, ToString},
    vec,
};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    api_error::ApiError,
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, NamedKeys},
    CLType, Parameter, RuntimeArgs, URef,
};
use casper_types::{runtime_args, CLValue};

const CONTRACT_QUESTION_KEY: &str = "dePoll_question";
const CONTRACT_OPTIONS_KEY: &str = "dePoll_options";
const CONTRACT_OPTIONS_DICT_UREF: &str = "dePoll_dict_seed_uref";
// const CONTRACT_VOTES_KEY: &str = "dePoll_votes";
const ACCESS_KEY: &str = "dePoll_contract_access_key";
const CONTRACT_HASH: &str = "dePoll_contract_hash";
const CONTRACT_PACKAGE: &str = "dePoll_contract_package";

const CONTRACT_VERSION_KEY: &str = "dePoll_version";
const INIT: &str = "init";
const INSTALLER: &str = "installer";

const RUNTIME_QUESTION_ARG: &str = "question";
const RUNTIME_OPTION_ONE_ARG: &str = "option_one";
const RUNTIME_OPTION_TWO_ARG: &str = "option_two";
const RUNTIME_ADD_OPTION_ARG: &str = "new_option";
const ENTRY_POINT_ADD_OPTION: &str = "add_option";
const ENTRY_POINT_VOTE: &str = "vote";
const RUNTIME_VOTE_ARG: &str = "vote_for";
const INITIAL_VOTE_COUNT: u64 = 0;

/// An error enum which can be converted to a `u16` so it can be returned as an `ApiError::User`.
#[repr(u16)]
#[allow(dead_code)]
enum Error {
    KeyAlreadyExists = 0,
    KeyMismatch = 1,
    InvalidVoteSubmission = 2,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        ApiError::User(error as u16)
    }
}

// // Define the struct for the voting contract
// #[derive(Default)]
// pub struct Poll {
//     options: Vec<String>,
//     votes: Vec<u64>,
// }

#[no_mangle]
pub extern "C" fn add_option() {
    let new_option: String = runtime::get_named_arg(RUNTIME_ADD_OPTION_ARG);
    // Get URef of dictionary.
    let options_dict_seed_uref: URef = runtime::get_key(CONTRACT_OPTIONS_KEY)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);

    match storage::dictionary_get::<u64>(options_dict_seed_uref, &new_option).unwrap_or_revert() {
        None => storage::dictionary_put(options_dict_seed_uref, &new_option, INITIAL_VOTE_COUNT),
        Some(_) => runtime::revert(Error::KeyAlreadyExists),
    }
}

#[no_mangle]
pub extern "C" fn vote() {
    let vote: String = runtime::get_named_arg(RUNTIME_VOTE_ARG);
    // Get the options dictionary seed URef
    let options_dict_seed_uref: URef = runtime::get_key(CONTRACT_OPTIONS_KEY)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);

    // Update the value of the vote option in the dictionary
    // storage::dictionary_put(options_dict_seed_uref, vote, option_value);
    match storage::dictionary_get::<u64>(options_dict_seed_uref, &vote).unwrap_or_revert() {
        None => runtime::revert(Error::InvalidVoteSubmission),

        Some(_) => {
            let old_option_value: u64 =
                storage::dictionary_get(options_dict_seed_uref, vote.clone().as_str())
                    .unwrap_or_revert_with(ApiError::Read)
                    .unwrap_or_revert_with(ApiError::ValueNotFound);
            let new_option_value: u64 = old_option_value + 1;
            storage::dictionary_put(options_dict_seed_uref, &vote, new_option_value);
        }
    }
}

#[no_mangle]
pub extern "C" fn init() {
    if runtime::get_key(CONTRACT_QUESTION_KEY).is_some() {
        runtime::revert(Error::KeyAlreadyExists)
    }

    let question: String = runtime::get_named_arg(RUNTIME_QUESTION_ARG);

    // Store new question
    let question_ref = storage::new_uref(question);
    runtime::put_key(CONTRACT_QUESTION_KEY, question_ref.into());

    let option_one: String = runtime::get_named_arg(RUNTIME_OPTION_ONE_ARG);
    let option_two: String = runtime::get_named_arg(RUNTIME_OPTION_TWO_ARG);

    // Store poll options
    let options_dict_seed_uref = storage::new_dictionary(CONTRACT_OPTIONS_KEY).unwrap_or_revert();
    match storage::dictionary_get::<u64>(options_dict_seed_uref, &option_one).unwrap_or_revert() {
        None => {
            storage::dictionary_put(options_dict_seed_uref, &option_one, INITIAL_VOTE_COUNT);
            storage::dictionary_put(options_dict_seed_uref, &option_two, INITIAL_VOTE_COUNT)
        }
        Some(_) => runtime::revert(Error::KeyAlreadyExists),
    }
    runtime::ret(CLValue::from_t(options_dict_seed_uref).unwrap_or_revert())
}

#[no_mangle]
pub extern "C" fn call() {
    // Create entry points for this contract
    let mut depoll_entry_points = EntryPoints::new();

    // Init Entrypoint
    depoll_entry_points.add_entry_point(EntryPoint::new(
        INIT,
        vec![
            Parameter::new(RUNTIME_QUESTION_ARG, CLType::String),
            Parameter::new(RUNTIME_OPTION_ONE_ARG, CLType::String),
            Parameter::new(RUNTIME_OPTION_TWO_ARG, CLType::String),
        ],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // Vote Submission Entrypoint
    depoll_entry_points.add_entry_point(EntryPoint::new(
        ENTRY_POINT_VOTE,
        vec![Parameter::new(RUNTIME_VOTE_ARG, CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // Add option Entrypoint
    depoll_entry_points.add_entry_point(EntryPoint::new(
        ENTRY_POINT_ADD_OPTION,
        vec![Parameter::new(RUNTIME_ADD_OPTION_ARG, CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    let named_keys = {
        let mut named_keys = NamedKeys::new();
        named_keys.insert(INSTALLER.to_string(), runtime::get_caller().into());
        named_keys
    };

    // Create a new contract package with various NamedKeys, applied contract package hash, and entrypoints.
    let (contract_hash, depoll_contract_version) = storage::new_contract(
        depoll_entry_points,
        Some(named_keys),
        Some(CONTRACT_PACKAGE.to_string()),
        Some(ACCESS_KEY.to_string()),
    );

    // Create a named key for the contract version
    let depoll_contract_version_ref = storage::new_uref(depoll_contract_version);
    runtime::put_key(CONTRACT_VERSION_KEY, depoll_contract_version_ref.into());

    // Create a named key for the contract hash
    runtime::put_key(CONTRACT_HASH, contract_hash.into());

    // Calls INIT entry point of the new contract (should be conditionned on upgrades)
    let options_dict_seed_uref = runtime::call_contract::<URef>(
        contract_hash,
        INIT,
        runtime_args! {
            RUNTIME_QUESTION_ARG => runtime::get_named_arg::<String>(RUNTIME_QUESTION_ARG),
            RUNTIME_OPTION_ONE_ARG => runtime::get_named_arg::<String>(RUNTIME_OPTION_ONE_ARG),
            RUNTIME_OPTION_TWO_ARG => runtime::get_named_arg::<String>(RUNTIME_OPTION_TWO_ARG),
        },
    );

    // Store dict seed uref in caller/installer context
    // This is not required, only information purpose
    runtime::put_key(CONTRACT_OPTIONS_DICT_UREF, options_dict_seed_uref.into());
}
