use super::snippet::{Snippet, SnippetInput, SnippetKey};
use candid::CandidType;
use candid::Deserialize;
use candid::Principal;
use ic_cdk::export::serde::de::DeserializeOwned;
use ic_kit::ic;
use ic_kit::ic::time;
use std::collections::BTreeMap;
use std::ops::Add;

pub enum AddSnippetResult {
    Added(String),
    Overflow((Page,String)),
}

#[derive( Clone, CandidType, Deserialize)]
pub struct Page {
    snippets: BTreeMap<SnippetKey, Snippet>,
    pub next_page: Option<String>,
    pub prev_page: Option<String>,
    pub max_size: usize,
    pub id: String,
}

impl Default for Page {
    fn default() -> Self {
        Page {
            snippets: BTreeMap::new(),
            next_page: None,
            prev_page: None,
            max_size: 10,
            id: "initial".to_string(),
        }
    }
}

impl Page {
    pub fn new(max_size: usize, id: String) -> Self {
        Self {
            snippets: Default::default(),
            next_page: None,
            prev_page: None,
            max_size,
            id,
        }
    }

    pub fn add_snippet(&mut self, snippet: SnippetInput, owner: Principal) -> AddSnippetResult {
        if self.snippets.len() > self.max_size {
            let mut new_page = Page::new(self.max_size, snippet.id.clone());
            let snippet_key = match new_page.add_snippet(snippet, owner) {
                AddSnippetResult::Added(key) => key,
             _=> panic!("should not happen"),
            };
            new_page.prev_page = Some(self.id.clone());
            self.next_page = Some(new_page.id.clone());
            return AddSnippetResult::Overflow((new_page, snippet_key));
        }

        let snippet_key = SnippetKey::new(snippet.id, self.id.clone());

        self.snippets.insert(
            snippet_key.clone(),
            Snippet {
                content: snippet.content.clone(),
                id: snippet_key.value.clone(),
                owner,
                timestamp: time(),
            },
        );

        AddSnippetResult::Added(snippet_key.value)
    }

    pub fn get_snippet(&self, id: &SnippetKey) -> Option<&Snippet> {
        self.snippets.get(id)
    }

    pub fn get_snippets(&self) -> Vec<&Snippet> {
        self.snippets.values().collect()
    }

    pub fn get_snippets_by_owner(&self, owner: &Principal) -> Vec<&Snippet> {
        self.snippets
            .values()
            .filter(|snippet| snippet.owner == *owner)
            .collect()
    }

    pub fn update_snippet(&mut self, snippet: SnippetInput) -> Option<Snippet> {
        self.snippets.insert(
            SnippetKey::new(snippet.id.clone(), self.id.clone()),
            snippet.to_snippet(ic::caller(), self.id.clone()),
        )
    }

    pub fn remove_snippet(&mut self, id: &str) -> Option<Snippet> {
        self.snippets
            .remove(&SnippetKey::new(id.to_string(), self.id.clone()))
    }
}
