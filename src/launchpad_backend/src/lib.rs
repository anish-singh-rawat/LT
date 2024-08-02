use candid::{CandidType, Deserialize, Principal, Nat};
use serde::Serialize;
use std::cell::RefCell;
use ic_cdk::update;
use ic_cdk::api::management_canister::main::CreateCanisterArgument;
use ic_cdk::api::management_canister::main::CanisterSettings as MgmtCanisterSettings;

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct Contract {
    token_id: String,
    receivers: Vec<Principal>,
    amount: Nat,
    duration: Nat,
    recurring: Nat,
    start_time: Nat,
}

#[derive(CandidType, Deserialize)]
pub struct CanisterSettings {
    freezing_threshold: Option<Nat>,
    controllers: Option<Vec<Principal>>,
    memory_allocation: Option<Nat>,
    compute_allocation: Option<Nat>,
}

#[derive(CandidType, Deserialize, Serialize, Clone)]
struct CanisterIdRecord {
    canister_id: Principal,
}

thread_local! {
    static CONTRACT_VERSION: RefCell<String> = RefCell::new("0.1.1".to_string());
    static INIT_CONTRACT_CYCLE: RefCell<Nat> = RefCell::new(Nat::from(300_000_000_000u64));
}

#[update]
async fn create_canister(settings: Option<CanisterSettings>) -> Result<Principal, String> {
    let settings = settings.map(|s| MgmtCanisterSettings {
        controllers: s.controllers,
        compute_allocation: s.compute_allocation,
        memory_allocation: s.memory_allocation,
        freezing_threshold: s.freezing_threshold,
        reserved_cycles_limit: Some(Nat::from(7_692_307_692u64)),
    });

    let arg = CreateCanisterArgument {
        settings,
    };

    let (canister_id_record,): (CanisterIdRecord,) = ic_cdk::api::call::call(
        Principal::management_canister(),
        "create_canister",
        (arg,),
    )
    .await
    .map_err(|e| format!("Failed to create canister: {}", e.1))?;

    Ok(canister_id_record.canister_id)
}

ic_cdk::export_candid!();
