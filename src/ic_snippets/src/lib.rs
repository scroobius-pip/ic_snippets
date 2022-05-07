mod page;
use candid::{CandidType, Deserialize, Principal};
use ic_kit::{ic, macros::*};
use page::page::{AddSnippetResult, Page};
use page::snippet::{Snippet, SnippetInput, SnippetKey};
use scaled_storage::{
    node::NodeResult,
    node_manager::{
        CanisterManager, CanisterManagerEvent, InitCanisterManagerParam, NodeInfo, WasmInitArgs,
    },
};

static mut CANISTER_MANAGER: Option<CanisterManager<Page>> = None;
static mut PAGE_ID: Option<String> = None;

#[derive(CandidType, Deserialize)]
pub struct UpdateResult {
    snippet_id: Option<String>,
    canister_id: Principal,
}

#[derive(CandidType, Deserialize)]
pub enum UpdateResponse {
    Ok(UpdateResult),
    Err(String),
}

#[derive(CandidType, Deserialize)]
pub struct Pagination {
    page: String,
    canister_id: Principal,
}

#[derive(CandidType, Deserialize)]
pub struct ListSnippets {
    next: Option<Pagination>,
    prev: Option<Pagination>,
    canister_id: candid::Principal,
    snippets: Vec<Snippet>,
}

#[derive(CandidType, Deserialize)]
pub enum ListSnippetsResponse {
    Ok(ListSnippetsResult),
    Err(String),
}

#[derive(CandidType, Deserialize)]
pub enum ListSnippetsResult {
    Snippets(Option<ListSnippets>),
    CanisterId(Principal),
}

#[derive(CandidType, Deserialize)]
pub enum GetSnippetResponse {
    Ok(GetSnippetResult),
    Err(String),
}

#[derive(CandidType, Deserialize)]
pub enum GetSnippetResult {
    Snippet(Option<Snippet>),
    CanisterId(Principal),
}

#[update]
pub async fn update_snippet(snippet_input: SnippetInput) -> UpdateResponse {
    let canister_manager = unsafe { CANISTER_MANAGER.as_mut().unwrap() };

    let snippet_key = match SnippetKey::from_string(snippet_input.id.clone()) {
        Ok(snippet_key) => snippet_key,
        Err(_) => return UpdateResponse::Err("Snippet key not found or invalid".to_string()),
    };

    let result = canister_manager
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
                (snippet_input,),
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
                    snippet_id: Some(snippet_input.id),
                    canister_id: ic::id(),
                }),
                Err(message) => UpdateResponse::Err(message.to_string()),
            }
        }
    }
}

#[update]
pub async fn add_snippet(snippet: SnippetInput) -> UpdateResponse {
    let canister_manager = unsafe { CANISTER_MANAGER.as_mut().unwrap() };
    let mut current_page_id = unsafe { PAGE_ID.clone().unwrap() };

    let result =
        canister_manager
            .canister
            .with_upsert_data_mut(current_page_id.to_string(), |page| {
                page.id = current_page_id.to_string();
                page.add_snippet(snippet.clone(), ic::caller())
            });

    match result {
        NodeResult::NodeId(canister_id) => {
            let result = CanisterManager::<Page>::forward_request::<UpdateResponse, _, _>(
                canister_id,
                "add_snippet",
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
                AddSnippetResult::Added(snippet_id) => UpdateResponse::Ok(UpdateResult {
                    snippet_id: Some(snippet_id),
                    canister_id: ic::id(),
                }),
                AddSnippetResult::Overflow(new_page) => {
                    let result = canister_manager.canister.with_upsert_data_mut(
                        new_page.0.id.clone(),
                        |page| {
                            let new_page_id = new_page.0.id.clone();
                            current_page_id = new_page_id;
                            *page = new_page.0.clone();
                            new_page.1
                        },
                    );

                    match result {
                        NodeResult::NodeId(canister_id) => {
                            let result =
                                CanisterManager::<Page>::forward_request::<UpdateResponse, _, _>(
                                    canister_id,
                                    "add_snippet",
                                    (snippet,),
                                )
                                .await;

                            match result {
                                Ok(response) => response,
                                Err(message) => UpdateResponse::Err(message),
                            }
                        }
                        NodeResult::Result(result) => UpdateResponse::Ok(UpdateResult {
                            snippet_id: Some(result.expect("Canister not found")),
                            canister_id: ic::id(),
                        }),
                    }
                }
            }
        }
    }
}

#[query]
pub fn get_snippet(id: String) -> GetSnippetResponse {
    let canister_manager = unsafe { CANISTER_MANAGER.as_mut().unwrap() };
    let snippet_key = match SnippetKey::from_string(id) {
        Ok(snippet_key) => snippet_key,
        Err(_) => return GetSnippetResponse::Err("Snippet key not found or invalid".to_string()),
    };

    let result = canister_manager
        .canister
        .with_data_mut(snippet_key.page_id(), |page| {
            let snippet = page.get_snippet(&snippet_key);
            snippet.map(|snippet| snippet.clone())
        });

    match result {
        NodeResult::NodeId(canister_id) => {
            GetSnippetResponse::Ok(GetSnippetResult::CanisterId(canister_id))
        }
        NodeResult::Result(result) => {
            let result = result.expect("Canister not found");
            match result {
                Some(snippet) => GetSnippetResponse::Ok(GetSnippetResult::Snippet(Some(snippet))),
                None => GetSnippetResponse::Ok(GetSnippetResult::Snippet(None)),
            }
        }
    }
}

#[query] 
pub fn list_snippets(page_id: String) -> ListSnippetsResponse {
    let canister_manager = unsafe { CANISTER_MANAGER.as_mut().unwrap() };

    let result = canister_manager
        .canister
        .with_data_mut(page_id, |page| page.clone());

    match result {
        NodeResult::NodeId(canister_id) => {
            ListSnippetsResponse::Ok(ListSnippetsResult::CanisterId(canister_id))
        }
        NodeResult::Result(result) => match result {
            Some(page) => {
                ListSnippetsResponse::Ok(ListSnippetsResult::Snippets(Some(ListSnippets {
                    snippets: page
                        .get_snippets()
                        .iter()
                        .map(|&snippet| snippet.clone())
                        .collect(),
                    canister_id: ic::id(),
                    next: match page.next_page {
                        Some(next_page) => Some(Pagination {
                            canister_id: match canister_manager
                                .canister
                                .with_data_mut(next_page.clone(), |_| ())
                            {
                                NodeResult::NodeId(canister_id) => canister_id,
                                _ => ic::id(),
                            },
                            page: next_page,
                        }),
                        None => None,
                    },
                    prev: match page.prev_page {
                        Some(prev_page) => Some(Pagination {
                            canister_id: match canister_manager
                                .canister
                                .with_data_mut(prev_page.clone(), |_| ())
                            {
                                NodeResult::NodeId(canister_id) => canister_id,
                                _ => ic::id(),
                            },
                            page: prev_page,
                        }),
                        None => None,
                    },
                })))
            }
            None => ListSnippetsResponse::Err("Canister not found".to_string()),
        },
    }
}

// Scaled Storage
//////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////

#[init]
fn init() {
    unsafe { CANISTER_MANAGER = Some(CanisterManager::new(ic::id(), |size| size > 2)) }
    unsafe {
        PAGE_ID = Some("initial".to_string());
    }
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

#[query]
fn node_info() -> NodeInfo {
    unsafe { CANISTER_MANAGER.as_mut().unwrap().node_info() }
}
