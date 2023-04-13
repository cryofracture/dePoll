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
use alloc::vec::Vec;

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    api_error::ApiError,
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, NamedKeys},
    CLType, Key, Parameter, URef, runtime_args, CLValue
};

use crate::runtime_args::RuntimeArgs;


// NamedKey and DictKey Values
const CONTRACT_QUESTION_KEY: &str = "dePoll_question";
const CONTRACT_KEY_OPTIONS: &str = "dePoll_options";
const CONTRACT_KEY_POLL_START: &str = "poll_start";
const CONTRACT_KEY_POLL_END: &str = "poll_end";
const CONTRACT_KEY_VERSION: &str = "version";
const CONTRACT_KEY_RESULTS: &str = "dePoll_results";

// Contract Variables
const CONTRACT_HASH: &str = "dePoll_contract_hash";
const CONTRACT_OPTIONS_DICT_REF: &str = "dePoll_dict_uref";
const INITIAL_VOTE_COUNT: u64 = 0;
const KEY_POLL_END: &str = "poll_end";
const INSTALLER: &str = "installer";

// Runtime Arguments
const RUNTIME_ARG_QUESTION: &str = "question";
const RUNTIME_ARG_OPTION_ONE: &str = "option_one";
const RUNTIME_ARG_OPTION_TWO: &str = "option_two";
const RUNTIME_ARG_ADD_OPTION: &str = "add_option";
const RUNTIME_ARG_CAST_VOTE: &str = "vote_for";


// Entrypoints
const ENTRY_POINT_VOTE: &str = "vote";
const ENTRY_POINT_ADD_OPTION: &str = "add_option";
const ENTRY_POINT_CLOSE_POLL: &str = "close_poll";
const ENTRY_POINT_INIT: &str = "init";

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

// #[no_mangle]
// pub extern "C" fn close_poll() {
//     let current_blocktime = u64::from(runtime::get_blocktime());
//     let options_dict_seed_uref: URef = runtime::get_key(CONTRACT_KEY_OPTIONS)
//         .unwrap_or_revert_with(ApiError::MissingKey)
//         .into_uref()
//         .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);
//
//     let poll_start_time_read = storage::dictionary_get::<u64>(dictionary, CONTRACT_KEY_POLL_START)
//         .unwrap_or_revert()
//         .unwrap_or_revert();
//
//     let poll_end_time_read = storage::dictionary_get::<u64>(dictionary, CONTRACT_KEY_POLL_END)
//         .unwrap_or_revert();
//
//     if current_blocktime >= poll_end_time_read {
//         let results_dict_seed_uref = storage::new_dictionary(CONTRACT_KEY_RESULTS).unwrap_or_revert();
//
//         // Get Current vote totals for all options in dePoll_options dict.
//     }
// }

#[no_mangle]
pub extern "C" fn init() {
    if runtime::get_key(CONTRACT_QUESTION_KEY).is_some() {
        runtime::revert(Error::KeyAlreadyExists)
    }

    // Get Poll Question and Options
    let question: String = runtime::get_named_arg(RUNTIME_ARG_QUESTION);
    let option_one: String = runtime::get_named_arg(RUNTIME_ARG_OPTION_ONE);
    let option_two: String = runtime::get_named_arg(RUNTIME_ARG_OPTION_TWO);
    let poll_start_time = u64::from(runtime::get_blocktime());
    let poll_end_time = poll_start_time + 5 * 60 * 1000; // add 5 minutes: 5 minutes of 60 seconds with 1000 milliseconds per second

    // Store new question
    let question_ref = storage::new_uref(question);
    runtime::put_key(CONTRACT_QUESTION_KEY, question_ref.into());

    // Store start and end times of poll
    let poll_start_ref = storage::new_uref(poll_start_time);
    let poll_end_ref = storage::new_uref(poll_end_time);
    runtime::put_key(CONTRACT_KEY_POLL_START, poll_start_ref.into());
    runtime::put_key(CONTRACT_KEY_POLL_END, poll_end_ref.into());

    let options_dict_seed_uref = storage::new_dictionary(CONTRACT_KEY_OPTIONS).unwrap_or_revert();
    // Compute poll_end time and store in dictionary


    match storage::dictionary_get::<u64>(options_dict_seed_uref, &option_one).unwrap_or_revert()
    {
        None => {
            storage::dictionary_put(options_dict_seed_uref, &option_one, INITIAL_VOTE_COUNT);
            storage::dictionary_put(options_dict_seed_uref, &option_two, INITIAL_VOTE_COUNT);
        }
        Some(_) => runtime::revert(Error::KeyAlreadyExists),
    }
    runtime::ret(CLValue::from_t(options_dict_seed_uref).unwrap_or_revert())
}

#[no_mangle]
pub extern "C" fn add_option() {
    let new_option: String = runtime::get_named_arg(RUNTIME_ARG_ADD_OPTION);

    let options_dict_seed_uref: URef = runtime::get_key(CONTRACT_KEY_OPTIONS)
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
    let new_vote: String = runtime::get_named_arg(RUNTIME_ARG_CAST_VOTE);
    // Get the options dictionary seed URef
    let options_dict_seed_uref: URef = runtime::get_key(CONTRACT_KEY_OPTIONS)
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
    // Create entry points for this contract
    let mut depoll_entry_points = EntryPoints::new();

    // Init Entrypoint
    depoll_entry_points.add_entry_point(EntryPoint::new(
        ENTRY_POINT_INIT,
        vec![
            Parameter::new(RUNTIME_ARG_QUESTION, CLType::String),
            Parameter::new(RUNTIME_ARG_OPTION_ONE, CLType::String),
            Parameter::new(RUNTIME_ARG_OPTION_TWO, CLType::String),
        ],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // Vote Submission Entrypoint
    depoll_entry_points.add_entry_point(EntryPoint::new(
        ENTRY_POINT_VOTE,
        vec![Parameter::new(RUNTIME_ARG_CAST_VOTE, CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // Entrypoint to add new option
    depoll_entry_points.add_entry_point(EntryPoint::new(
        ENTRY_POINT_ADD_OPTION,
        vec![Parameter::new(RUNTIME_ARG_ADD_OPTION, CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    // Entrypoint to close the poll and disable voting
    // depoll_entry_points.add_entry_point(EntryPoint::new(
    //     ENTRY_POINT_CLOSE_POLL,
    //     Vec::new(),
    //     CLTpe::Unit,
    //     EntryPointAccess::Public,
    //     EntryPointType::Contract,
    // ));

    let mut depoll_named_keys = NamedKeys::new();
    let poll_start_time = u64::from(runtime::get_blocktime());
    let poll_end_time = poll_start_time + 5 * 60 * 1000; // add 5 minutes: 5 minutes of 60 seconds with 1000 milliseconds per second

    // Create new URefs for namedkeys
    let poll_start_ref = storage::new_uref(poll_start_time);
    let poll_end_ref = storage::new_uref(poll_end_time);

    // Create new Keys
    let poll_start_key = Key::URef(poll_start_ref);
    let poll_end_key = Key::URef(poll_end_ref);

    // Put Keys to Contract context
    depoll_named_keys.insert(CONTRACT_KEY_POLL_START.to_string(), poll_start_key.into());
    depoll_named_keys.insert(CONTRACT_KEY_POLL_END.to_string(), poll_end_key.into());
    // depoll_named_keys.insert(INSTALLER.to_string(), runtime::get_caller().into());


    // Create a new contract package with various NamedKeys, applied contract package hash, and entrypoints.
    let (depoll_contract_hash, depoll_contract_version_hash) = storage::new_contract(
        depoll_entry_points,
        Some(depoll_named_keys),
        Some("dePoll_contract_package".to_string()),
        Some("dePoll_contract_access_key".to_string()),
    );

    // Calls INIT entry point of the new contract (should be conditionned on upgrades)
    let options_dict_seed_uref = runtime::call_contract::<URef>(
        depoll_contract_hash,
        ENTRY_POINT_INIT,
        runtime_args! {
            RUNTIME_ARG_QUESTION => runtime::get_named_arg::<String>(RUNTIME_ARG_QUESTION),
            RUNTIME_ARG_OPTION_ONE => runtime::get_named_arg::<String>(RUNTIME_ARG_OPTION_ONE),
            RUNTIME_ARG_OPTION_TWO => runtime::get_named_arg::<String>(RUNTIME_ARG_OPTION_TWO),
        },
    );

    let depoll_contract_uref = storage::new_uref(depoll_contract_hash);
    let depoll_contract_key = Key::URef(depoll_contract_uref);
    let options_dict_seed_ref = storage::new_uref(options_dict_seed_uref);
    let options_dict_seed_key = Key::URef(options_dict_seed_ref);

    // Put the NamedKey values.
    runtime::put_key(CONTRACT_KEY_OPTIONS, options_dict_seed_key);
    runtime::put_key(CONTRACT_HASH, depoll_contract_key);

    // Store dict seed uref in caller/installer context
    // This is not required, only information purpose
    runtime::put_key(CONTRACT_OPTIONS_DICT_REF, options_dict_seed_uref.into());
}
