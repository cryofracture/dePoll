#!/bin/bash

# Define the session hash and entry point
SESSION_HASH="hash-62eb9af7a7ceb7e51a0f1b69721b3f78180b5ff2fdd3f68cc524538d7fd92e88"
SESSION_ENTRY_POINT="vote"

# Define the options to vote for
OPTIONS=("Anthony Volpe" "Trea Turner" "Cedric Mullins II" "Starling Marte")

# Loop over the keys and vote a random number of times out of 100 for each option
for DIR in ~/keys/cryo_int{3,6,7,9}/; do
  KEY="${DIR}CryoIntegrationTest${DIR: -2:1}_secret_key.pem"
  for OPTION in "${OPTIONS[@]}"; do
    # Generate a random number of votes between 1 and 25
    VOTES=$((1 + RANDOM % 5))
    echo "Voting $VOTES times for $OPTION using key $KEY"

    # Submit vote for each candidate for each number of votes
    for ((i=1; i<=$VOTES; i++)); do
      casper-client put-deploy -n https://rpc.integration.casperlabs.io \
        --chain-name integration-test \
        --payment-amount 1000000000 \
        -k "${KEY}" \
        --session-hash "${SESSION_HASH}" \
        --session-entry-point "${SESSION_ENTRY_POINT}" \
        --session-arg "vote_for:string='${OPTION}'"
    done
  done
done
