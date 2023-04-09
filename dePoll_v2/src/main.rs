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
    CLType, Key, Parameter, URef
};

const CONTRACT_QUESTION_KEY: &str = "dePoll_question";
const CONTRACT_OPTIONS_KEY: &str = "dePoll_options";
const CONTRACT_HASH: &str = "dePoll_contract_hash";
const CONTRACT_OPTIONS_DICT_REF: &str = "dePoll_dict_uref";
const CONTRACT_VERSION_KEY: &str = "version";
const RUNTIME_QUESTION_ARG: &str = "question";
const RUNTIME_OPTION_ONE_ARG: &str = "option_one";
const RUNTIME_OPTION_TWO_ARG: &str = "option_two";
const RUNTIME_ADD_OPTION_ARG: &str = "add_option";
const ENTRY_POINT_VOTE: &str = "vote";
const ENTRY_POINT_ADD_OPTION: &str = "add_option";
const RUNTIME_VOTE_ARG: &str = "vote_for";
const INITIAL_VOTE_COUNT: u64 = 0;

/// An error enum which can be converted to a `u16` so it can be returned as an `ApiError::User`.
#[repr(u16)]
#[allow(dead_code)]
enum Error {
    KeyAlreadyExists = 0,
    KeyMismatch = 1,
    InvalidVoteSubmission = 2,
    InvalidNewPollOption = 3,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        ApiError::User(error as u16)
    }
}

#[no_mangle]
pub extern "C" fn add_option() {
    let new_option: String = runtime::get_named_arg(RUNTIME_ADD_OPTION_ARG);

    let options_dict_seed_uref: URef = runtime::get_key(CONTRACT_OPTIONS_KEY)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);

    match storage::dictionary_get::<u64>(options_dict_seed_uref, &new_option).unwrap_or_revert()
    {
        None => storage::dictionary_put(options_dict_seed_uref, &new_option, INITIAL_VOTE_COUNT),
        Some(_) => runtime::revert(Error::InvalidNewPollOption),
    }
}

#[no_mangle]
pub extern "C" fn vote() {
    let new_vote: String = runtime::get_named_arg(RUNTIME_VOTE_ARG);
    // Get the options dictionary seed URef
    let options_dict_seed_uref: URef = runtime::get_key(CONTRACT_OPTIONS_KEY)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);

    let old_option_value: u64 =
        storage::dictionary_get(options_dict_seed_uref, &new_vote)
            .unwrap_or_revert_with(ApiError::Read)
            .unwrap_or_revert_with(ApiError::ValueNotFound);
    let new_option_value: u64 = old_option_value + 1;

    // Update the value of the vote option in the dictionary
    match storage::dictionary_get::<u64>(options_dict_seed_uref, &new_vote).unwrap_or_revert() {
        None => runtime::revert(Error::InvalidVoteSubmission),
        Some(_) => storage::dictionary_put(options_dict_seed_uref, &new_vote, new_option_value),
    }
}

#[no_mangle]
pub extern "C" fn call() {
    // Get Poll Question and Options
    let question: String = runtime::get_named_arg(RUNTIME_QUESTION_ARG);
    let option_one: String = runtime::get_named_arg(RUNTIME_OPTION_ONE_ARG);
    let option_two: String = runtime::get_named_arg(RUNTIME_OPTION_TWO_ARG);

    let options = vec![option_one, option_two];

    let options_dict_seed_uref = storage::new_dictionary(CONTRACT_OPTIONS_KEY).unwrap_or_revert();

    for option in options {
        match storage::dictionary_get::<u64>(options_dict_seed_uref, &option).unwrap_or_revert()
        {
            None => storage::dictionary_put(options_dict_seed_uref, &option, INITIAL_VOTE_COUNT),
            Some(_) => runtime::revert(Error::KeyAlreadyExists),
        }
    }

    let mut depoll_named_keys = NamedKeys::new();

    // Create new URefs for namedkeys
    let options_seed_uref = storage::new_uref(options_dict_seed_uref);
    let question_ref = storage::new_uref(question);

    // Create new Keys
    let depoll_dict_key = Key::URef(options_seed_uref);
    let depoll_question_key = Key::URef(question_ref);

    // Put Keys to Contract context
    depoll_named_keys.insert(CONTRACT_QUESTION_KEY.to_string(), depoll_question_key);
    depoll_named_keys.insert(CONTRACT_OPTIONS_KEY.to_string(), options_dict_seed_uref.into());

    // Create entry points for this contract
    let mut depoll_entry_points = EntryPoints::new();

    // Vote Submission Entrypoint
    depoll_entry_points.add_entry_point(EntryPoint::new(
        ENTRY_POINT_VOTE,
        vec![Parameter::new(RUNTIME_VOTE_ARG, CLType::String)],
        CLType::String,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // Entrypoint to add new option
    depoll_entry_points.add_entry_point(EntryPoint::new(
        ENTRY_POINT_ADD_OPTION,
        vec![Parameter::new(RUNTIME_ADD_OPTION_ARG, CLType::String)],
        CLType::String,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // Create a new contract package with various NamedKeys, applied contract package hash, and entrypoints.
    let (depoll_contract_hash, depoll_contract_version_hash) = storage::new_contract(
        depoll_entry_points,
        Some(depoll_named_keys),
        Some("dePoll_package_hash".to_string()),
        Some("dePoll_contract_access_uref".to_string()),
    );

    let depoll_contract_uref = storage::new_uref(depoll_contract_hash);
    let depoll_contract_version_ref = storage::new_uref(depoll_contract_version_hash);
    let depoll_contract_key = Key::URef(depoll_contract_uref);
    let depoll_version_key = Key::URef(depoll_contract_version_ref);
    let options_dict_seed_ref = storage::new_uref(options_dict_seed_uref);
    let options_dict_seed_key = Key::URef(options_dict_seed_ref);
    // depoll_named_keys.insert(CONTRACT_VERSION_KEY.to_string(), depoll_version_key);
    // depoll_named_keys.insert(CONTRACT_HASH.to_string(), depoll_contract_key);

    // Put the NamedKey values.
    // runtime::put_key(CONTRACT_QUESTION_KEY, depoll_question_key);
    runtime::put_key(CONTRACT_VERSION_KEY, depoll_version_key);
    runtime::put_key(CONTRACT_OPTIONS_KEY, depoll_dict_key);
    runtime::put_key(CONTRACT_HASH, depoll_contract_key);
    runtime::put_key(CONTRACT_OPTIONS_DICT_REF, options_dict_seed_key);
}
