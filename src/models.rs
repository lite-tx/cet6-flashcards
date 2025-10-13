use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Word {
    pub word: String,
    #[serde(default)]
    pub translations: Vec<Translation>,
    #[serde(default)]
    pub phrases: Vec<Phrase>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Translation {
    pub translation: String,
    #[serde(rename = "type", default)]
    pub word_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Phrase {
    pub phrase: String,
    pub translation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyProgress {
    pub current_index: usize,
    pub mastered: Vec<String>,
    pub difficult: Vec<String>,
}

impl Default for StudyProgress {
    fn default() -> Self {
        Self {
            current_index: 0,
            mastered: Vec::new(),
            difficult: Vec::new(),
        }
    }
}
