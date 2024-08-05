use candid::{CandidType, Deserialize, Principal};
use ic_cdk_macros::{update, query};
use ic_cdk::api::{call::call, call::RejectionCode, canister_balance128};

#[derive(CandidType, Deserialize, Clone)]
struct ArchiveOptions {
    num_blocks_to_archive: u64,
    max_transactions_per_response: Option<u64>,
    trigger_threshold: u64,
    max_message_size_bytes: Option<u64>,
    cycles_for_archive_creation: Option<u64>,
    node_max_memory_size_bytes: Option<u64>,
    controller_id: Principal,
    more_controller_ids: Option<Vec<Principal>>,
}

pub type Subaccount = Vec<u8>;

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct Account {
    owner: Principal,
    subaccount: Option<Subaccount>,
}

#[derive(CandidType, Deserialize, Clone)]
struct InitArgsSimplified {
    decimals: Option<u8>,
    maximum_number_of_accounts: Option<u64>,
    accounts_overflow_trim_quantity: Option<u64>,
    feature_flags: Option<FeatureFlags>,
    metadata: Vec<(String, MetadataValue)>,
}

#[derive(CandidType, Deserialize, Clone)]
struct InitArgsRequested {
    token_symbol: String,
    token_name: String,
    logo: String,
    transfer_fee: u64,
    fee_collector_account: Option<Principal>,
}

#[derive(CandidType, Deserialize, Clone)]
struct FeatureFlags {
    icrc2: bool,
}

#[derive(CandidType, Deserialize, Clone)]
enum MetadataValue {
    Text(String),
}

#[derive(CandidType, Deserialize, Clone)]
struct LedgerArg {
    init_args: Option<InitArgsSimplified>,
    upgrade_args: Option<UpgradeArgs>,
    fee_collector_account: Account,
}

#[derive(CandidType, Deserialize, Clone)]
enum ChangeFeeCollector {
    SetTo(Principal),
    Unset,
}

#[derive(CandidType, Deserialize, Clone)]
struct UpgradeArgs {
    token_symbol: Option<String>,
    transfer_fee: Option<u64>,
    metadata: Option<Vec<(String, MetadataValue)>>,
    maximum_number_of_accounts: Option<u64>,
    accounts_overflow_trim_quantity: Option<u64>,
    change_fee_collector: Option<ChangeFeeCollector>,
    max_memo_length: Option<u16>,
    token_name: Option<String>,
    feature_flags: Option<FeatureFlags>,
}

static SNS_WASM_VERSION: &[u8; 64] = b"af8fc1469e553ac90f704521a97a1e3545c2b68049b4618a6549171b4ea4fba8";
static CYCLES_FOR_ARCHIVE: u64 = 300_000_000_000;
static CYCLES_FOR_INSTALL: u128 = 300_000_000_000;
static MIN_CYCLES_IN_DEPLOYER: u128 = 2_000_000_000_000;
static CREATION_FEE: u64 = 100_000_000;


#[update]
async fn transfer_ownership(canister_id: Principal, to: Principal) -> Result<(), String> {
    let result: Result<(), (RejectionCode, String)> = call(
        Principal::management_canister(),
        "update_settings",
        (UpdateSettings {
            canister_id,
            settings: CanisterSettings {
                controllers: Some(vec![to]),
                compute_allocation: None,
                memory_allocation: None,
                freezing_threshold: Some(9_331_200),
                reserved_cycles_limit: None,
            },
            sender_canister_version: None,
        },),
    )
    .await;
    result.map_err(|(_, e)| format!("Failed to transfer ownership: {}", e))
}


#[derive(CandidType, Deserialize)]
struct UpdateSettings {
    canister_id: Principal,
    settings: CanisterSettings,
    sender_canister_version: Option<u64>,
}

#[derive(CandidType, Deserialize)]
struct CanisterSettings {
    controllers: Option<Vec<Principal>>,
    compute_allocation: Option<u64>,
    memory_allocation: Option<u64>,
    freezing_threshold: Option<u64>,
    reserved_cycles_limit: Option<u64>,
}

#[query]
async fn get_canister_balance() -> u128 {
    canister_balance128()
}

#[update]
async fn install(req_args: InitArgsRequested) -> Result<Principal, String> {
    if req_args.token_symbol.len() > 8 {
        return Err("Token symbol too long, max 8 characters".to_string());
    }
    if req_args.token_name.len() > 32 {
        return Err("Token name too long, max 32 characters".to_string());
    }
    if req_args.logo.len() < 100 {
        return Err("Logo too small".to_string());
    }
    if req_args.logo.len() > 30_000 {
        return Err("Max logo size is 20 KB".to_string());
    }

    let init_args = InitArgsSimplified {
        decimals: Some(8),
        maximum_number_of_accounts: Some(28_000_000),
        accounts_overflow_trim_quantity: Some(100_000),
        feature_flags: Some(FeatureFlags { icrc2: true }),
        metadata: vec![("icrc1:logo".to_string(), MetadataValue::Text(req_args.logo.clone()))],
    };

    let archive_options = ArchiveOptions {
        num_blocks_to_archive: 1000,
        trigger_threshold: 2000,
        node_max_memory_size_bytes: Some(1024 * 1024 * 1024),
        max_message_size_bytes: Some(128 * 1024),
        cycles_for_archive_creation: Some(CYCLES_FOR_ARCHIVE),
        controller_id: ic_cdk::id(),
        max_transactions_per_response: None,
        more_controller_ids: Some(vec![ic_cdk::id()]),
    };

    let args = LedgerArg {
        init_args: Some(init_args.clone()),
        upgrade_args: None,
        fee_collector_account: Account {
            owner: req_args.fee_collector_account.unwrap_or(ic_cdk::id()),
            subaccount: None,
        },
    };

    let upgradearg = UpgradeArgs {
        token_symbol: Some(req_args.token_symbol.clone()),
        transfer_fee: Some(req_args.transfer_fee),
        metadata: Some(init_args.metadata.clone()),
        maximum_number_of_accounts: init_args.maximum_number_of_accounts,
        accounts_overflow_trim_quantity: init_args.accounts_overflow_trim_quantity,
        max_memo_length: Some(80),
        token_name: Some(req_args.token_name.clone()),
        feature_flags: init_args.feature_flags.clone(),
        change_fee_collector: Some(ChangeFeeCollector::SetTo(req_args.fee_collector_account.unwrap_or(ic_cdk::id()))),
    };

    let balance = get_canister_balance().await;
    if balance < CYCLES_FOR_INSTALL + MIN_CYCLES_IN_DEPLOYER {
        return Err(format!("Not enough cycles in deployer, balance: {}", balance));
    }

    if !_is_admin(ic_cdk::caller()) {
        let res: Result<(), (RejectionCode, String)> = call(
            ic_cdk::id(),
            "icrc2_transfer_from",
            (
                TransferFromArgs {
                    from: From {
                        owner: ic_cdk::caller(),
                        subaccount: None,
                    },
                    spender_subaccount: None,
                    to: From {
                        owner: deployer(),
                        subaccount: None,
                    },
                    fee: None,
                    memo: None,
                    from_subaccount: None,
                    created_at_time: None,
                    amount: CREATION_FEE,
                },
            ),
        )
        .await;
        
        if let Err((_, e)) = res {
            return Err(format!("Transfer from failed: {}", e));
        }
    }

    let canister_id = create_canister().await?;
    Ok(canister_id)
}


async fn create_canister() -> Result<Principal, String> {
    let result: Result<(Principal,), (RejectionCode, String)> = call(
        Principal::management_canister(),
        "create_canister",
        ()
    )
    .await;

    result
        .map_err(|(_, e)| format!("Failed to create canister: {}", e))
        .map(|(canister_id,)| canister_id)
}



fn _is_admin(caller: Principal) -> bool {
    true
}

#[derive(CandidType, Deserialize)]
struct TransferFromArgs {
    from: From,
    spender_subaccount: Option<Principal>,
    to: From,
    fee: Option<u64>,
    memo: Option<Vec<u8>>,
    from_subaccount: Option<Vec<u8>>,
    created_at_time: Option<u64>,
    amount: u64,
}

#[derive(CandidType, Deserialize)]
struct From {
    owner: Principal,
    subaccount: Option<Vec<u8>>,
}

fn deployer() -> Principal {
    Principal::anonymous()
}

ic_cdk::export_candid!();
