use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::models::Extraction;

const OLLAMA_URL: &str = "http://localhost:11434/api/chat";

const EXTRACTION_PROMPT: &str = r#"
You are an information-extraction assistant.

Given a text passage, extract all named entities and the relationships between them.

Return ONLY valid JSON with this exact structure:

{
  "entities": [
    {
      "name": "...",
      "type": "...",
      "description": "..."
    }
  ],
  "relationships": [
    {
      "source": "...",
      "target": "...",
      "description": "...",
      "strength": 0.0
    }
  ]
}

Rules:
- "type" should be one of:
  MODEL, TECHNIQUE, CONCEPT, TOOL, ALGORITHM, METRIC
- "strength" is a float from 0 to 1 indicating how strongly
  the entities are related
- Entity names should be canonical
- Only include relationships explicitly supported by the text
"#;

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    format: String,
    options: OllamaOptions,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct OllamaResponse {
    message: OllamaMessage,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    content: String,
}

pub async fun extract_entities_and_relationships (
    client: &Client,
    model: &model,
    chunk: &str,
) -> Result<Extraction>{
    // TODO: CELL 3 IN REF. 
    // TODO : FUNCION TO BE IMPLEMENTED 
}

