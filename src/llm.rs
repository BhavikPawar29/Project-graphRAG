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

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    content: String,
}

pub async fn extract_entities_and_relationships ( client: &Client, model: &str, chunk: &str) -> Result<Extraction>{
  let request = OllamaRequest {
    model: model.to_string(),
    messages: vec![
      Message{
        role: "system".to_string(),
        content: EXTRACTION_PROMPT.to_string(),
      },
      Message{
        role: "user".to_string(),
        content:format!("Text:\n{}", chunk),
      },
    ],

    stream: false,

    format: "json".to_string(),

    options: OllamaOptions {
            temperature: 0.0,
        },
  };

  let response = client.post(OLLAMA_URL).json(&request).send().await.context("Failed to contact Ollama")?;

  let response = response.error_for_status().context("Ollama returned an error")?;

  let ollama_response: OllamaResponse = response.json().await.context("Failed to parse Ollama reponse")?;

  let extraction: Extraction = serde_json::from_str(&ollama_response.message.content).context("LLM returned invalied extraction JSON")?;

  Ok(extraction)
}

