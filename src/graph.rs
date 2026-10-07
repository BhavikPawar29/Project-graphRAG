use petgraph::graph::{Graph, NodeIndex};
use petgraph::Undirected;
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};

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

pub struct KnowledgeGraph {
    pub graph: Graph<GraphNode, GraphEdge, Undirected>,
    pub node_indices: HashMap<String, NodeIndex>,
}

#[derive(Debug)]
pub struct Community {
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

    pub fn group_by_community(&self, labels:  &[usize]) -> Vec<Community> {
        let mut groups: HashMap<usize, Vec<NodeIndex>> = HashMap::new();

        for node_index in self.graph.node_indices() {
            let community_id = labels[node_index.index()];

            groups.entry(community_id).or_default().push(node_index);
        }

        groups.into_iter().map(|(id, members)| Community {id, members}).collect()
    }

    pub fn build_community_content(&self, community: &Community) -> String {
        let mut content = String::new();

        content.push_str("Entities:\n");

        for node_index in &community.members {
            let node = &self.graph[*node_index];

            content.push_str(&format!(
                 "- {} ({}): {}\n",
                    node.name,
                    node.entity_type,
                    node.description
            ));
        }

        let member_set: HashSet<NodeIndex> = community.members.iter().copied().collect();

        let mut relationship_lines = Vec::new();

        for edge in self.graph.edge_references() {
            let source = edge.source();
            let target = edge.target();

            if member_set.contains(&source) || member_set.contains(&target) {
                let source_node = &self.graph[source];
                let target_node = &self.graph[target];
                let relationship = edge.weight();

                relationship_lines.push(format!(
                    "- {} -> {}: {}",
                    source_node.name,
                    target_node.name,
                    relationship.description
                ));
            }
        }

        if !relationship_lines.is_empty() {
            content.push_str("\nRelationships:\n");

            for line in relationship_lines {
                content.push_str(&line);
                content.push('\n');
            }
        }

        content

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
        let source = petgraph::graph::NodeIndex::new(source);
        let target = petgraph::graph::NodeIndex::new(target);

        self.graph
            .find_edge(source, target)
            .map(|edge_index| {self.
                graph[edge_index].strength as f64
            })
            .unwrap_or(0.0)
    }
}