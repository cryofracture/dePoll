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

    CLType, Key, Parameter, URef, runtime_args, CLValue,
    account::AccountHash,
};

use crate::runtime_args::RuntimeArgs;


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


/// An error enum which can be converted to a `u16` so it can be returned as an `ApiError::User(Error)`.
#[repr(u16)]
#[allow(dead_code)]
enum Error {
    KeyAlreadyExists = 0,
    KeyMismatch = 1,
    InvalidVoteSubmission = 2,
    InvalidNewPollOption = 3,
    PollNoLongerOpen = 4,
    UnauthorizedRequest = 5,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        ApiError::User(error as u16)
    }
}

#[no_mangle]
pub extern "C" fn init() {
    if runtime::get_key(CONTRACT_QUESTION_KEY).is_some() {
        runtime::revert(Error::KeyAlreadyExists)
    }

    // Get Poll Question and Options
    let question: String = runtime::get_named_arg(RUNTIME_ARG_QUESTION);
    let option_one: String = runtime::get_named_arg(RUNTIME_ARG_OPTION_ONE);
    let option_two: String = runtime::get_named_arg(RUNTIME_ARG_OPTION_TWO);

    // Gets contract's context and retrieves the poll_start u64 value
    let poll_start_ref: URef = runtime::get_key(CONTRACT_KEY_POLL_START)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);
    let poll_start_time: u64 = storage::read(poll_start_ref)
        .unwrap_or_revert_with(ApiError::Read)
        .unwrap_or_revert_with(ApiError::ValueNotFound);

    // Gets contract's context and retrieves the poll_end u64 value
    let poll_end_ref: URef = runtime::get_key(CONTRACT_KEY_POLL_END)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);
    let poll_end_time: u64 = storage::read(poll_end_ref)
        .unwrap_or_revert_with(ApiError::Read)
        .unwrap_or_revert_with(ApiError::ValueNotFound);

    // Store new question
    let question_ref = storage::new_uref(question);

    let options_dict_seed_uref = storage::new_dictionary(CONTRACT_KEY_OPTIONS).unwrap_or_revert();
    // Compute poll_end time and store in dictionary

    let option_count:u8 = 2;

    let option_one_ref = storage::new_uref(&*option_one);
    let option_two_ref = storage::new_uref(&*option_two);
    let option_count_ref = storage::new_uref(option_count);

    let option_count_key = Key::URef(option_count_ref);

    runtime::put_key(CONTRACT_QUESTION_KEY, question_ref.into());
    runtime::put_key(CONTRACT_OPTIONS_DICT_UREF, options_dict_seed_uref.into());
    runtime::put_key(CONTRACT_KEY_OPTION_ONE, option_one_ref.into());
    runtime::put_key(CONTRACT_KEY_OPTION_TWO, option_two_ref.into());
    runtime::put_key(CONTRACT_KEY_OPTIONS_COUNT, option_count_key);


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
pub extern "C" fn extend_poll() {
    let current_blocktime = u64::from(runtime::get_blocktime());
    let poll_end_ref: URef = runtime::get_key(CONTRACT_KEY_POLL_END)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);
    let poll_end_time: u64 = storage::read(poll_end_ref)
        .unwrap_or_revert_with(ApiError::Read)
        .unwrap_or_revert_with(ApiError::ValueNotFound);

    let poll_extension_length: u64 = runtime::get_named_arg(RUNTIME_ARG_EXTEND_POLL);
    let new_poll_end_time: u64 = poll_end_time + poll_extension_length * SECONDS_PER_MIN * MILLI_PER_SEC; // add 5 minutes: 5 minutes of 60 seconds with 1000 milliseconds per second


    if current_blocktime <= poll_end_time {
        storage::write(poll_end_ref, new_poll_end_time);
    } else if current_blocktime > poll_end_time {
        runtime::revert(Error::PollNoLongerOpen)
    }
}

#[no_mangle]
pub extern "C" fn add_option() {
    let new_option: String = runtime::get_named_arg(RUNTIME_ARG_ADD_OPTION);
    let current_blocktime = u64::from(runtime::get_blocktime());

    let option_count_ref: URef = runtime::get_key(CONTRACT_KEY_OPTIONS_COUNT)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);

    let old_option_count: u8 = storage::read(option_count_ref)
        .unwrap_or_revert_with(ApiError::Read)
        .unwrap_or_revert_with(ApiError::ValueNotFound);

    let new_option_count: u8 = old_option_count + 1;

    let old_option_count_str = &old_option_count.to_string();
    let new_option_count_str = &new_option_count.to_string();

    let new_option_key: String = CONTRACT_KEY_OPTION_X.to_string() + new_option_count_str;
    let new_option_ref = storage::new_uref(&*new_option_key);

    runtime::put_key(&new_option_key, new_option_ref.into());

    storage::write(option_count_ref, new_option_count);

    let poll_end_ref: URef = runtime::get_key(CONTRACT_KEY_POLL_END)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);

    let poll_end_time: u64 = storage::read(poll_end_ref)
        .unwrap_or_revert_with(ApiError::Read)
        .unwrap_or_revert_with(ApiError::ValueNotFound);

    let options_dict_seed_uref: URef = runtime::get_key(CONTRACT_KEY_OPTIONS)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);

    let poll_owner: AccountHash = runtime::get_key(INSTALLER)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_account()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);

    let caller = runtime::get_caller();

    if caller == poll_owner {
        if current_blocktime < poll_end_time {
            match storage::dictionary_get::<u64>(options_dict_seed_uref, &new_option).unwrap_or_revert()
            {
                None => storage::dictionary_put(options_dict_seed_uref, &new_option, INITIAL_VOTE_COUNT),
                Some(_) => runtime::revert(Error::InvalidNewPollOption),
            }
        }
    } else { runtime::revert(Error::UnauthorizedRequest)}
}

#[no_mangle]
pub extern "C" fn vote() {
    let current_blocktime = u64::from(runtime::get_blocktime());
    let poll_end_ref: URef = runtime::get_key(CONTRACT_KEY_POLL_END)
        .unwrap_or_revert_with(ApiError::MissingKey)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);
    let poll_end_time: u64 = storage::read(poll_end_ref)
        .unwrap_or_revert_with(ApiError::Read)
        .unwrap_or_revert_with(ApiError::ValueNotFound);


    if current_blocktime <= poll_end_time {
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

    } else { runtime::revert(Error::PollNoLongerOpen) }
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
            Parameter::new(RUNTIME_ARG_POLL_LENGTH, CLType::String),
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

    depoll_entry_points.add_entry_point(EntryPoint::new(
        ENTRY_POINT_EXTEND_POLL,
        vec![Parameter::new(RUNTIME_ARG_EXTEND_POLL, CLType::String)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    let mut depoll_named_keys = NamedKeys::new();
    let poll_start_time = u64::from(runtime::get_blocktime());
    let poll_length: u64 = runtime::get_named_arg(RUNTIME_ARG_POLL_LENGTH);
    let poll_end_time: u64 = poll_start_time + poll_length * SECONDS_PER_MIN * MILLI_PER_SEC; // add 5 minutes: 5 minutes of 60 seconds with 1000 milliseconds per second

    // Create new URefs for namedkeys
    let poll_start_ref = storage::new_uref(poll_start_time);
    let poll_end_ref = storage::new_uref(poll_end_time);

    // Create new Keys
    let poll_start_key = Key::URef(poll_start_ref);
    let poll_end_key = Key::URef(poll_end_ref);

    // Put Keys to Contract context
    depoll_named_keys.insert(CONTRACT_KEY_POLL_START.to_string(), poll_start_key.into());
    depoll_named_keys.insert(CONTRACT_KEY_POLL_END.to_string(), poll_end_key.into());
    depoll_named_keys.insert(INSTALLER.to_string(), runtime::get_caller().into());


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
    // let options_dict_seed_ref = storage::new_uref(options_dict_seed_uref);
    // let options_dict_seed_key = Key::URef(options_dict_seed_ref);

    // Put the NamedKey values.
    // runtime::put_key(CONTRACT_KEY_OPTIONS, options_dict_seed_key);
    runtime::put_key(CONTRACT_HASH, depoll_contract_key);

    // Store dict seed uref in caller/installer context
    // This is not required, only information purpose
    runtime::put_key(CONTRACT_OPTIONS_DICT_UREF, options_dict_seed_uref.into());
}
