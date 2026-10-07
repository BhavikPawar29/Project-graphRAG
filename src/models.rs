use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Extraction {
    pub entities: Vec<Entity>,
    pub relationships: Vec<Relationship>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub name: String, 

    #[serde(rename = "type")]
    pub entity_type: String,
    pub description: String,   
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Relationship {
    pub source: String,
    pub target: String,
    pub description: String,
    pub strength: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComunityReport {
    pub title: String,
    pub summary: String,
    pub key_themes: Vec<String>,
    pub most_important_entities: Vec<String>,
    pub community_id: usize,
    pub members: Vec<String>,
}

