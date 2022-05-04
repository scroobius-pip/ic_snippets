use std::collections::BTreeMap;
use std::ops::Add;
use ic_cdk::export::serde::de::DeserializeOwned;
use ic_kit::ic;
use ic_kit::ic::time;
use candid::CandidType;
use candid::Deserialize;
use candid::Principal;
use super::snippet::{Snippet,SnippetKey,SnippetInput};

pub enum AddSnippetResult {
    Added,
    Overflow(Page),
}

#[derive(Default, Clone, CandidType, Deserialize)]
pub struct Page {
    snippets: BTreeMap<SnippetKey, Snippet>,
    pub next_page: Option<String>,
    pub prev_page: Option<String>,
    pub max_size: usize,
    pub id: u32,
}

impl Page {
    pub fn new(max_size: usize, id: u32) -> Self {
        Self {
            snippets: Default::default(),
            next_page: None,
            prev_page: None,
            max_size,
            id,
        }
    }

    pub fn add_snippet(&mut self, snippet: SnippetInput, owner: Principal) -> AddSnippetResult {
        if self.snippets.len() >= self.max_size {
            let new_page = Page::new(self.max_size, self.id.add(1));
            return AddSnippetResult::Overflow(new_page);
        }

        let snippet_key = SnippetKey::new(snippet.id, self.id.clone());

        self.snippets.insert(
            snippet_key.clone(),
            Snippet {
                content: snippet.content,
                id: snippet_key.value,
                owner,
                timestamp: time(),
            },
        );

        AddSnippetResult::Added
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