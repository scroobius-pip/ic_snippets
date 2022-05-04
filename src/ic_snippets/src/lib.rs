mod page;
use candid::{CandidType, Deserialize, Principal};
use ic_kit::{ic,macros::*};
use page::page::Page;
use page::snippet::Snippet;
use page::snippet::SnippetInput;
use scaled_storage::node_manager::{
    CanisterManager, CanisterManagerEvent, InitCanisterManagerParam, WasmInitArgs,
};

static mut CANISTER_MANAGER: Option<CanisterManager<Page>> = None;

#[derive(CandidType, Deserialize)]
struct UpdateResult {
    result: bool,
    canister_id: Principal,
}

#[derive(CandidType, Deserialize)]
struct Pagination {
    page: String,
    canister_id: Principal,
}

#[derive(CandidType, Deserialize)]
struct ListSnippetResult {
    next: Option<Pagination>,
    prev: Option<Pagination>,
    canister_id: candid::Principal,
    snippets: Vec<Snippet>,
}



pub fn update_snippet( snippet: SnippetInput) -> UpdateResult {
 
    let mut canister_manager = unsafe { CANISTER_MANAGER.as_mut().unwrap() };
    

}














// Scaled Storage
//////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////

#[init]
fn init() {
    unsafe { CANISTER_MANAGER = Some(CanisterManager::new(ic::id(), |size| size > 1000)) }
}

#[update]
fn init_wasm(param: WasmInitArgs) -> bool {
    unsafe {
        CANISTER_MANAGER
            .as_mut()
            .unwrap()
            .lifecycle_init_wasm(param)
    }
}

#[heartbeat]
async fn heartbeat() {
    unsafe {
        CANISTER_MANAGER
            .as_mut()
            .unwrap()
            .lifecyle_heartbeat_node()
            .await;
    }
}

#[update]
async fn handle_event(event: CanisterManagerEvent) {
    unsafe {
        CANISTER_MANAGER
            .as_mut()
            .unwrap()
            .lifecycle_handle_event(event)
            .await
    }
}

#[update]
async fn init_canister_manager(param: InitCanisterManagerParam) {
    unsafe {
        match param.args {
            Some(args) => CANISTER_MANAGER
                .as_mut()
                .unwrap()
                .lifecyle_init_node(Some(args.all_nodes)),
            None => CANISTER_MANAGER.as_mut().unwrap().lifecyle_init_node(None),
        }
        .await
    }
}
