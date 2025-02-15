export PRE_MINTED_TOKENS=10_000_000_000
dfx identity use default
export DEFAULT=$(dfx identity get-principal)
export TRANSFER_FEE=10_000
dfx identity new archive_controller
dfx identity use archive_controller
export ARCHIVE_CONTROLLER=$(dfx identity get-principal)
export TRIGGER_THRESHOLD=2000
export CYCLE_FOR_ARCHIVE_CREATION=10000000000000
export NUM_OF_BLOCK_TO_ARCHIVE=1000
export TOKEN_NAME="My Token"
export TOKEN_SYMBOL="XMTK"
dfx identity new minter
dfx identity use minter
export MINTER=$(dfx identity get-principal)

# If you only want to support the ICRC-1 standard, then you can set the flag to false. If you want to also support the ICRC-2 standard, set it to true.

export FEATURE_FLAGS=false