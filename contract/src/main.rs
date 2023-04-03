#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use alloc::vec::Vec;
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
const CONTRACT_OPTIONS_DICT_REF: &str = "dePoll_dict_hash";
const CONTRACT_VOTES_KEY: &str = "dePoll_votes";
const CONTRACT_HASH: &str = "dePoll_contract_hash";
const CONTRACT_PACKAGE_HASH: &str = "dePoll_contract_package_hash";
const CONTRACT_VERSION_KEY: &str = "version";
const RUNTIME_QUESTION_ARG: &str = "question";
const RUNTIME_OPTION_ONE_ARG: &str = "option_one";
const RUNTIME_OPTION_TWO_ARG: &str = "option_two";
const RUNTIME_ADD_OPTION_ARG: &str = "add_option";
const ENTRY_POINT_VOTE: &str = "vote";
const RUNTIME_VOTE_ARG: &str = "vote_for";
const ENTRY_POINT_INIT: &str = "initialize";
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

// Define the struct for the voting contract
#[derive(Default)]
pub struct Poll {
    options: Vec<String>,
    votes: Vec<u64>,
}

#[no_mangle]
pub extern "C" fn add_option(new_option: String) {
    // Get URef of dictionary.
    let poll_options_uref: URef = runtime::get_key(CONTRACT_OPTIONS_KEY)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);

    match storage::dictionary_get::<u64>(poll_options_uref, &new_option)
        .unwrap_or_revert()
    {
        None => storage::dictionary_put(poll_options_uref, &new_option, 0u64),
        Some(_) => runtime::revert(Error::KeyAlreadyExists),
    }
}

#[no_mangle]
pub extern "C" fn vote(vote: String) {
    // Get the options dictionary seed URef
    let options_dict_seed_uref: URef = runtime::get_key(CONTRACT_OPTIONS_DICT_REF)
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
pub extern "C" fn call() {
    // Get Poll Question and Options
    let question: String = runtime::get_named_arg(RUNTIME_QUESTION_ARG);
    let option_one: String = runtime::get_named_arg(RUNTIME_OPTION_ONE_ARG);
    let option_two: String = runtime::get_named_arg(RUNTIME_OPTION_TWO_ARG);



    let mut depoll_named_keys = NamedKeys::new();

    // Set URefs for new question
    let question_ref = storage::new_uref(question);
    // Create a new Key for NamedKeys
    let question_key = Key::URef(question_ref);
    // Put the NamedKey value.
    runtime::put_key(CONTRACT_QUESTION_KEY, question_key);

    // Create a new Poll instance and call its init function with the options argument
    let options_dict_seed_uref = storage::new_dictionary(CONTRACT_OPTIONS_KEY).unwrap_or_revert();
    // runtime::put_key(CONTRACT_OPTIONS_DICT_REF, casper_types::Key::URef(options_dict_seed_uref));
    // Add poll option one
    match storage::dictionary_get::<u64>(options_dict_seed_uref, &option_one).unwrap_or_revert()
    {
        None => storage::dictionary_put(options_dict_seed_uref, &option_one, INITIAL_VOTE_COUNT),
        Some(_) => runtime::revert(Error::KeyAlreadyExists),
    }

    // Add poll option two
    match storage::dictionary_get::<u64>(options_dict_seed_uref, &option_two).unwrap_or_revert()
    {
        None => storage::dictionary_put(options_dict_seed_uref, &option_two, INITIAL_VOTE_COUNT),
        Some(_) => runtime::revert(Error::KeyAlreadyExists),
    }

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

    // Create a new contract package with various NamedKeys, applied contract package hash, and entrypoints.
    let (depoll_contract_package_hash, depoll_contract_version_hash) = storage::new_contract(
        depoll_entry_points,
        Some(depoll_named_keys),
        Some(CONTRACT_PACKAGE_HASH.to_string()),
        Some("dePoll_contract_package_hash".to_string()),
    );

    let options_seed_uref = storage::new_uref(options_dict_seed_uref);

    let depoll_options_dict_seed_key = Key::URef(options_seed_uref);
    runtime::put_key(CONTRACT_OPTIONS_DICT_REF, depoll_options_dict_seed_key);

    // Create new URefs for contract hashes
    let depoll_contract_version_ref = storage::new_uref(depoll_contract_version_hash);
    let depoll_contract_package_ref = storage::new_uref(depoll_contract_package_hash);

    // Create a new Key for NamedKeys
    let contract_version_key = Key::URef(depoll_contract_version_ref);
    let contract_package_hash_key = Key::URef(depoll_contract_package_ref);

    // Create a NamedKey for the contract version
    runtime::put_key(CONTRACT_VERSION_KEY, contract_version_key);

    // Create a named key for the contract package hash
    runtime::put_key(CONTRACT_PACKAGE_HASH, contract_package_hash_key);
}
