mod graph;
mod llm;
mod models;

use graph::KnowledgeGraph;

use std::env;

use anyhow::{Ok, Result};
use dotenvy::dotenv;
use petgraph::graph::Graph;
use petgraph::visit::EdgeRef;
use reqwest::Client;

use serde::{Deserialize, Serialize};


use llm::extract_entities_and_relationships;

#[tokio::main]
async fn main() -> Result<()>{
    // dotenv().ok();
    
    // let api_key = env::var("key");
    
    // println!("API key loaded.");
    // println!("Setup complete.");

    // Ok(())

    let corpus = vec![
         r#"Transformer models use self-attention to relate all positions in a sequence
        simultaneously. The attention mechanism computes query, key, and value projections,
        then applies scaled dot-product attention. BERT introduced bidirectional
        pre-training, while GPT models use causal (left-to-right) attention masks."#,

        r#"Large language models are typically trained with the next-token prediction
        objective on massive text corpora. GPT-4 and Claude use reinforcement learning
        from human feedback (RLHF) to align model outputs with human preferences.
        Constitutional AI from Anthropic adds a self-critique step before RLHF."#,

        r#"Retrieval-augmented generation (RAG) combines a dense retrieval system with
        a generative language model. A query is encoded into an embedding, the nearest
        chunks are retrieved from a vector store such as FAISS or Pinecone, and the
        context is prepended to the LLM prompt. RAG reduces hallucination and allows
        knowledge to be updated without retraining."#,

        r#"Knowledge graphs represent facts as (subject, predicate, object) triples.
        Neo4j is a popular property graph database. SPARQL is used to query RDF graphs.
        Entity linking maps surface mentions to canonical graph nodes, and relation
        extraction populates the graph from unstructured text."#,

        r#"Quantisation reduces model size by representing weights in lower precision.
        INT8 and INT4 are common targets. GPTQ applies one-shot post-training
        quantisation using approximate second-order information. AWQ activates
        weight quantisation to preserve salient weights. Both methods keep accuracy
        close to the full-precision baseline."#,

        r#"The KV cache stores the key and value tensors from previous attention
        computations so they need not be recomputed at each decoding step. This trades
        memory for compute. For a 70B-parameter model with long contexts, the KV cache
        can exceed the size of the model weights themselves, making it a primary
        bottleneck for batch throughput."#,

        r#"Community detection algorithms partition graph nodes into groups with dense
        internal connections. The Louvain algorithm optimises modularity greedily.
        Leiden improves on Louvain by guaranteeing well-connected communities and
        avoiding poorly connected splits. Both algorithms are widely used for
        social network analysis and knowledge graph clustering."#,

        r#"Vector databases such as Weaviate, Qdrant, and Chroma store dense embeddings
        and support approximate nearest-neighbour search via HNSW or IVF indexing.
        They underpin semantic search, recommendation engines, and RAG pipelines.
        Hybrid search combines dense retrieval with sparse BM25 scoring."#,
    ];

    let model = "qwen3:4b-instruct";

    let client = Client::new();

    println!("Extracting entities and relationships...\n");

    let mut total_entities = 0;
    let mut total_relationships = 0;

    let mut extractions = Vec::new();

    for (i, chunk) in corpus.iter().enumerate(){
        println!("Processing chunk {}...", i + 1);

        let extraction = 
            extract_entities_and_relationships(
                &client, 
                model, 
                chunk
            )
            .await?;

        let entity_count = extraction.entities.len();
        let relationship_count = extraction.relationships.len();

        total_entities += entity_count;
        total_relationships += relationship_count;

         println!(
            "  Chunk {}: {} entities, {} relationships",
            i + 1,
            entity_count,
            relationship_count
        );

        extractions.push(extraction);
    }

    println!();
    println!(
        "Total before merging: {} entities, {} relationships",
        total_entities,
        total_relationships
    );

    let knowledge_graph = KnowledgeGraph::build(&extractions);

    println!(
        "\nGraph: {} nodes, {} edges",
        knowledge_graph.graph.node_count(),
        knowledge_graph.graph.edge_count()
    );

    println!("\nKnowledge Graph:");

    for node_index in knowledge_graph.graph.node_indices() {
        let node = &knowledge_graph.graph[node_index];

        println!(
            "\n{} [{}]",
            node.name,
            node.entity_type
        );

        println!("  {}", node.description);

        for edge in knowledge_graph.graph.edges(node_index){
            let target_index = edge.target();
            let target = &knowledge_graph.graph[target_index];
            let relationship = edge.weight();

            println!(
                "  -- {} ({:.2}) --> {}",
                relationship.description,
                relationship.strength,
                target.name
            );
        }
    }

    Ok(())

}