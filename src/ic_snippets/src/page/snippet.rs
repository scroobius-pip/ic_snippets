use candid::Principal;
use candid::{CandidType, Deserialize};
use ic_kit::ic::time;

#[derive(Clone, CandidType, PartialEq, Eq, Debug, Deserialize)]
pub struct SnippetKey {
    pub value: String,
}

impl SnippetKey {
    pub fn new(snippet_id: String, page_id: String) -> Self {
        Self {
            value: format!("{}_{}", snippet_id, page_id),
        }
    }

    pub fn from_string(s: String) -> Result<Self, ()> {
        let parts: Vec<&str> = s.split("_").collect();
        if parts.len() != 2 {
            return Err(());
        }
        let snippet_id = parts[0].to_string();

        let page_id = parts[1].to_string();
        Ok(Self::new(snippet_id, page_id))
    }

    pub fn page_id(&self) -> String {
        self.value.split("_").last().unwrap().to_string()
    }
}

impl Ord for SnippetKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for SnippetKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, CandidType, PartialEq, Eq, Deserialize)]
pub struct Snippet {
    pub content: String,
    pub owner: Principal,
    pub id: String,
    pub timestamp: u64,
}

impl Ord for Snippet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl PartialOrd for Snippet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, CandidType, Deserialize)]
pub struct SnippetInput {
    pub content: String,
    pub id: String,
}

impl SnippetInput {
    pub fn to_snippet(self, owner: Principal, page_id: String) -> Snippet {
        let snippet_key = SnippetKey::new(self.id, page_id);
        Snippet {
            content: self.content,
            id: snippet_key.value,
            owner,
            timestamp: time(),
        }
    }
}
