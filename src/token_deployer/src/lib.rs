use candid::{CandidType, Deserialize, Principal, Nat};
use serde::Serialize;
use std::cell::RefCell;
use ic_cdk::{update,caller};
use ic_cdk::api::management_canister::main::{CreateCanisterArgument ,CanisterSettings as MgmtCanisterSettings,UpdateSettingsArgument ,CanisterIdRecord};
use ic_cdk::api::call::call_with_payment;

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

thread_local! {
    static CONTRACT_VERSION: RefCell<String> = RefCell::new("0.1.1".to_string());
    static INIT_CONTRACT_CYCLE: RefCell<Nat> = RefCell::new(Nat::from(300_000_000_000u64));
    static CONTRACTS: RefCell<Vec<String>> = RefCell::new(Vec::new());
    static ADMINS: RefCell<Vec<Principal>> = RefCell::new(vec![Principal::from_text("jfzrt-43ntz-fu3jt-wvhqy-c726z-gxadc-s47rt-ckxnp-3m32a-5umd7-2ae").unwrap()]);
}


fn _is_admin(caller: Principal) -> bool {
    ADMINS.with(|admins| admins.borrow().contains(&caller))
}


#[update]
async fn create_canister(settings: Option<CanisterSettings>) -> Result<Principal, String> {
    let settings: Option<MgmtCanisterSettings> = settings.map(|s: CanisterSettings| MgmtCanisterSettings {
        controllers: s.controllers,
        compute_allocation: s.compute_allocation,
        memory_allocation: s.memory_allocation,
        freezing_threshold: s.freezing_threshold,
        reserved_cycles_limit: Some(Nat::from(7_692_307_692u64)),
    });

    let arg: CreateCanisterArgument = CreateCanisterArgument {
        settings,
    };

    let (canister_id_record,): (CanisterIdRecord,) = ic_cdk::api::call::call(
        Principal::management_canister(),
        "create_canister",
        (arg,),
    )
    .await
    .map_err(|e: (ic_cdk::api::call::RejectionCode, String)| format!("Failed to create canister: {}", e.1))?;

    Ok(canister_id_record.canister_id)
}


#[update]
async fn create_contract(contract: Contract) -> Result<String, String> {
    let controllers = vec![Principal::from_text("jfzrt-43ntz-fu3jt-wvhqy-c726z-gxadc-s47rt-ckxnp-3m32a-5umd7-2ae").unwrap()];
    let init_cycles: u64 = INIT_CONTRACT_CYCLE.with(|cycle| cycle.borrow().clone().0.to_u64_digits().first().copied().unwrap_or_default());

    let (new_contract_canister,): (Principal,) = call_with_payment(
        Principal::management_canister(),
        "create_canister",
        (CreateCanisterArgument {
            settings: Some(MgmtCanisterSettings {
                controllers: Some(controllers.clone()),
                compute_allocation: Some(contract.amount.clone()),
                memory_allocation: Some(contract.duration.clone()),
                freezing_threshold: Some(contract.recurring.clone()),
                reserved_cycles_limit: None,
            }),
        },),
        init_cycles, 
    ).await.map_err(|e: (ic_cdk::api::call::RejectionCode, String)| e.1)?;

    CONTRACTS.with(|contracts| {
        contracts.borrow_mut().push(new_contract_canister.to_text());
    });

    Ok(new_contract_canister.to_text())
}



#[update]
async fn add_controller(canister_id: Principal, controllers: Vec<Principal>) -> Result<(), String> {
    let caller = caller();
    if !_is_admin(caller) {
        return Err("Caller is not an admin".to_string());
    }

    let update_arg = UpdateSettingsArgument {
        canister_id: canister_id,
        settings: MgmtCanisterSettings {
            controllers: Some(controllers),
            compute_allocation: Some(Nat::from(0u32)),
            memory_allocation: Some(Nat::from(0u32)),
            freezing_threshold: Some(Nat::from(2_592_000u32)),            
            reserved_cycles_limit: None,
        }
    };

    ic_cdk::api::call::call(
        Principal::management_canister(),
        "update_settings",
        (update_arg,),
    )
    .await
    .map_err(|e: (ic_cdk::api::call::RejectionCode, String)| format!("Failed to update settings: {}", e.1))?;

    Ok(())
}


ic_cdk::export_candid!();
