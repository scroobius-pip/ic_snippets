mod page;
use candid::{CandidType, Deserialize, Principal};
use ic_kit::{ic, macros::*};
use page::page::Page;
use page::snippet::Snippet;
use page::snippet::SnippetInput;
use page::snippet::SnippetKey;
use scaled_storage::node::NodeResult;
use scaled_storage::node_manager::{
    CanisterManager, CanisterManagerEvent, InitCanisterManagerParam, WasmInitArgs,
};

static mut CANISTER_MANAGER: Option<CanisterManager<Page>> = None;
static mut PAGE_ID: Option<String> = Some("initial".to_string());

#[derive(CandidType, Deserialize)]
struct UpdateResult {
    value: bool,
    canister_id: Principal,
}

#[derive(CandidType, Deserialize)]
enum UpdateResponse {
    Ok(UpdateResult),
    Err(String),
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

pub async fn update_snippet(snippet: SnippetInput) -> UpdateResponse {
    let mut canister_manager = unsafe { CANISTER_MANAGER.as_mut().unwrap() };

    let SnippetInput { content, id } = snippet.clone();

    let snippet_key = match SnippetKey::from_string(id) {
        Ok(snippet_key) => snippet_key,
        Err(_) => return UpdateResponse::Err("Snippet key not found or invalid".to_string()),
    };

    let result =
        canister_manager
            .canister
            .with_upsert_data_mut(snippet_key.page_id(), |page| {
                match page.get_snippet(&snippet_key) {
                    Some(snippet) => {
                        let owner = snippet.owner;
                        if owner != ic::caller() {
                            return Err("Auth Error");
                        }
                        Ok(true)
                    }
                    None => Err("Could not find snippet"),
                }
            });

    match result {
        NodeResult::NodeId(canister_id) => {
            let result = CanisterManager::<Page>::forward_request::<UpdateResponse, _, _>(
                canister_id,
                "update_snippet",
                (snippet,),
            )
            .await;

            match result {
                Ok(response) => response,
                Err(message) => UpdateResponse::Err(message),
            }
        }
        NodeResult::Result(result) => {
            let result = result.expect("Canister not found");
            match result {
                Ok(response) => UpdateResponse::Ok(UpdateResult {
                    value: response,
                    canister_id: ic::id(),
                }),
                Err(message) => UpdateResponse::Err(message.to_string()),
            }
        },
    }
}

pub fn add_snippet(snippet: SnippetInput) -> UpdateResult {
    let mut canister_manager = unsafe { CANISTER_MANAGER.as_mut().unwrap() };
    let page_id = unsafe { PAGE_ID.unwrap() };

    let result = canister_manager
        .canister
        .with_upsert_data_mut(page_id.to_string(), |page| {
            page.add_snippet(snippet.clone(), ic::caller());
        });
}

// Scaled Storage
//////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////

#[init]
fn init() {
    unsafe { CANISTER_MANAGER = Some(CanisterManager::new(ic::id(), |size| size > 1000)) }
    unsafe { PAGE_ID = Some(0) }
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
