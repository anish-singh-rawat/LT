MINTER=$(dfx --identity default identity get-principal)
DEFAULT=$(dfx --identity default identity get-principal)
RECIEVER=$(dfx --identity reciever identity get-principal)
TOKEN_SYMBOL=TOK
TOKEN_NAME="DAOTOKEN"
TRANSFER_FEE=1000
FEATURE_FLAGS=true
PRE_MINTED_TOKENS=100000000000
echo $RECIEVER
NUM_OF_BLOCK_TO_ARCHIVE=100;
TRIGGER_THRESHOLD=100;
CYCLE_FOR_ARCHIVE_CREATION=true

dfx deploy icrc1_ledger_canister --argument "(variant {Init = 
record {
     token_symbol = \"${TOKEN_SYMBOL}\";
     token_name = \"${TOKEN_NAME}\";
     minting_account = record { owner = principal \"${MINTER}\" };
     transfer_fee = ${TRANSFER_FEE};
     metadata = vec {};
     initial_balances = vec { record { record { owner = principal \"${DEFAULT}\"; }; ${PRE_MINTED_TOKENS}; }; };
     archive_options = record {
         num_blocks_to_archive = 100;
         trigger_threshold = 100;
         controller_id = principal \"${DEFAULT}\";
     };
     feature_flags = opt record {icrc2 = true;};
 }
})"

dfx deploy launchpad_backend
dfx deploy launchpad_frontend 