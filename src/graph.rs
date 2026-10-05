use petgraph::graph::{Graph, NodeIndex};
use petgraph::Undirected;
use std::collections::HashMap;
use std::vec;

use graphops::graph::GraphRef;
use graphops::louvain::louvain_seeded;

use graphops::graph::{Graph as GraphOpsGraph, WeightedGraph};

use crate::models::{Entity, Extraction, Relationship};


#[derive(Debug, Clone)]
pub struct GraphNode {
    pub name: String,
    pub entity_type: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub description: String,
    pub strength: f32, 
    pub source_chunk_id: usize,
}

struct CommunityGraph {
    neighbors: Vec<Vec<usize>>,
}

impl GraphRef for CommunityGraph {
    fn node_count(&self) -> usize {
        self.neighbors.len()
    }

    fn neighbors_ref(&self, node: usize) -> &[usize] {
        &self.neighbors[node]
    }
    
}

pub struct KnowledgeGraph {
    pub graph: Graph<GraphNode, GraphEdge, Undirected>,
    pub node_indices: HashMap<String, NodeIndex>,
}

pub struct Commmunity {
    pub id: usize, 
    pub members: Vec<NodeIndex>,
}

impl KnowledgeGraph {
    pub fn new () -> Self {
        Self {
            graph: Graph::new_undirected(),
            node_indices: HashMap::new(),
        }
    }

    pub fn add_entity(&mut self, entity: &Entity) -> NodeIndex{

        if let Some(&node_index) = self.node_indices.get(&entity.name) {
             let node = &mut self.graph[node_index];

             if !entity.description.is_empty() && !node.description.contains(&entity.description) {
                if !node.description.is_empty() {
                    node.description.push(' ');
                }

                node.description.push_str(&entity.description);
             }
            return node_index;
        }

        let node = GraphNode {
            name: entity.name.clone(),
            entity_type: entity.entity_type.clone(),
            description: entity.description.clone(),
        };

        let node_index = self.graph.add_node(node);

        self.node_indices.insert(entity.name.clone(), node_index);

        node_index

    }

    pub fn add_relationship(&mut self, relationship: &Relationship, source_chunk_id: usize) {
        let Some(&source) = self.node_indices.get(&relationship.source) else {
            return;
        };

        let Some(&target) = self.node_indices.get(&relationship.target) else {
            return;
        };

        if let Some(edge_index) = self.graph.find_edge(source, target) {
            let edge = &mut self.graph[edge_index];

            edge.strength = (edge.strength + relationship.strength).min(1.0);

            if !edge.description.contains(&relationship.description) {
                edge.description.push(' ');
                edge.description.push_str(&relationship.description);
            }
        }
        else {
            let edge = GraphEdge {
                description: relationship.description.clone(),
                strength: relationship.strength,
                source_chunk_id,
            };

            self.graph.add_edge(source, target, edge);
        }
    }

    pub fn build(extractions: &[Extraction]) -> Self {
        let mut knowledge_graph = Self::new();

        //First add every entity
        for extraction in extractions {
            for entity in &extraction.entities {
                knowledge_graph.add_entity(entity);
            }
        }

        // Then add the realtionships
        for (chunk_id, extraction) in extractions.iter().enumerate() {
            for relationship in &extraction.relationships {
                knowledge_graph.add_relationship(relationship, chunk_id);
            }
        }

        knowledge_graph
    }

    pub fn detect_communities(&self) -> Vec<usize> {
        let weighted_graph = WeightedPetgraph {
            graph: &self.graph,
        };

        graphops::louvain::louvain_weighted(&weighted_graph, 1.0)
    }
    

}

pub struct WeightedPetgraph<'a> {
    pub graph: &'a petgraph::Graph<GraphNode, GraphEdge, petgraph::Undirected>,
}

impl <'a> GraphOpsGraph for WeightedPetgraph<'a> {

    fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    fn neighbors(&self, node: usize) -> Vec<usize> {
        self.graph.neighbors(petgraph::graph::NodeIndex::new(node)).map(|idx| idx.index()).collect()
    }
    
}

impl<'a> WeightedGraph for WeightedPetgraph<'a> {
    fn edge_weight(&self, source: usize, target: usize) -> f64 {
        let source = petgraph::graph::node_index(source);
        let target = petgraph::graph::node_index(target);

        self.graph.find_edge(source, target).map(|edge_index| self.graph[edge_index].strength as f64).unwrap_or(0.0)
    }
}