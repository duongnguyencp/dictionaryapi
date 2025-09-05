use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Phonetic {
    pub text: Option<String>,
    pub audio: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Definition {
    pub definition: String,
    pub example: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Meaning {
    pub part_of_speech: String,
    pub definitions: Vec<Definition>,
}

#[derive(Serialize, Deserialize)]
pub struct WordEntry {
    pub word: String,
    pub phonetics: Option<Vec<Phonetic>>,
    pub meanings: Vec<Meaning>,
}
