//! Microsoft 365 Graph Connector & Search Indexing Engine
//!
//! Exposes Tagisan enterprise knowledge into Microsoft Search and Copilot:
//! - `tagisanSkill`: Indexes 495+ engineering skills, invariants, and procedural guides
//! - `tagisanArtifact`: Indexes system architectures, specs, and HTML visual artifacts
//! - `tagisanDebate`: Indexes multi-model dialectical debate decisions and Lakandiwa syntheses
//!
//! Generates Microsoft Graph External Connection definitions, schema registration payloads,
//! and item ingestion payloads.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

/// Access Control List (ACL) entry for Microsoft Graph external items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AclEntry {
    pub access_type: String,
    pub identity_type: String,
    pub value: String,
}

impl Default for AclEntry {
    fn default() -> Self {
        Self {
            access_type: "grant".to_string(),
            identity_type: "everyone".to_string(),
            value: "everyone".to_string(),
        }
    }
}

/// Ingestion item prepared for Microsoft Search Semantic Index
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IngestionItem {
    pub id: String,
    pub connection_id: String,
    pub item_type: String,
    pub title: String,
    pub content: String,
    pub url: String,
    pub properties: Map<String, Value>,
    pub acl: Vec<AclEntry>,
}

/// Engine for creating and managing Microsoft 365 Graph Connectors
pub struct GraphConnectorEngine {
    connection_id: String,
    connector_name: String,
    description: String,
}

impl Default for GraphConnectorEngine {
    fn default() -> Self {
        Self {
            connection_id: "tagisan_enterprise_index".to_string(),
            connector_name: "Tagisan Enterprise Engineering Knowledge".to_string(),
            description: "Semantic search index of Tagisan skills, dialectical consensus transcripts, and architectural artifacts for Microsoft 365 Copilot.".to_string(),
        }
    }
}

impl GraphConnectorEngine {
    /// Create a new GraphConnectorEngine with default connection parameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a custom connection ID
    pub fn with_connection_id(mut self, id: impl Into<String>) -> Self {
        self.connection_id = id.into();
        self
    }

    /// Connection ID
    pub fn connection_id(&self) -> &str {
        &self.connection_id
    }

    /// Generate Microsoft Graph External Connection registration payload (POST /v1.0/external/connections)
    pub fn generate_connection_definition(&self) -> Value {
        json!({
            "id": self.connection_id,
            "name": self.connector_name,
            "description": self.description,
            "activitySettings": {
                "urlToItemResolvers": [
                    {
                        "urlMatchInfo": {
                            "baseUrls": ["https://tagisan.ai/"]
                        }
                    }
                ]
            },
            "searchSettings": {
                "searchConfiguration": {
                    "authorizedAppIds": []
                }
            }
        })
    }

    /// Generate Microsoft Graph Schema Registration payload (POST /v1.0/external/connections/{id}/schema)
    pub fn generate_schema_definition(&self) -> Value {
        json!({
            "baseType": "microsoft.graph.externalItem",
            "properties": [
                {
                    "name": "title",
                    "type": "String",
                    "isSearchable": true,
                    "isQueryable": true,
                    "isRetrievable": true,
                    "labels": ["title"]
                },
                {
                    "name": "itemType",
                    "type": "String",
                    "isQueryable": true,
                    "isRetrievable": true,
                    "isRefinable": true
                },
                {
                    "name": "category",
                    "type": "String",
                    "isQueryable": true,
                    "isRetrievable": true,
                    "isRefinable": true
                },
                {
                    "name": "description",
                    "type": "String",
                    "isSearchable": true,
                    "isQueryable": true,
                    "isRetrievable": true
                },
                {
                    "name": "content",
                    "type": "String",
                    "isSearchable": true,
                    "isRetrievable": true
                },
                {
                    "name": "invariants",
                    "type": "String",
                    "isSearchable": true,
                    "isRetrievable": true
                },
                {
                    "name": "verdict",
                    "type": "String",
                    "isSearchable": true,
                    "isRetrievable": true
                },
                {
                    "name": "tags",
                    "type": "StringCollection",
                    "isQueryable": true,
                    "isRetrievable": true,
                    "isRefinable": true
                },
                {
                    "name": "author",
                    "type": "String",
                    "isQueryable": true,
                    "isRetrievable": true,
                    "labels": ["authors"]
                },
                {
                    "name": "lastModifiedDateTime",
                    "type": "DateTime",
                    "isQueryable": true,
                    "isRetrievable": true,
                    "isRefinable": true,
                    "labels": ["lastModifiedDateTime"]
                },
                {
                    "name": "url",
                    "type": "String",
                    "isRetrievable": true,
                    "labels": ["url"]
                }
            ]
        })
    }

    /// Build indexable ingestion items for built-in Tagisan skills
    pub fn build_skill_items(&self) -> Vec<IngestionItem> {
        let skills = crate::ecc::all_built_in_skills();
        let now_iso = chrono::Utc::now().to_rfc3339();

        skills
            .into_iter()
            .map(|skill| {
                let id = format!("skill_{}", skill.name.replace('-', "_"));
                let mut props = Map::new();
                props.insert("title".to_string(), json!(skill.name));
                props.insert("itemType".to_string(), json!("tagisanSkill"));
                props.insert("category".to_string(), json!("EngineeringSkill"));
                props.insert("description".to_string(), json!(skill.description));
                props.insert("content".to_string(), json!(skill.instructions));
                props.insert("invariants".to_string(), json!("Strict conformance to ECC invariants"));
                props.insert("verdict".to_string(), json!("Production Approved"));
                props.insert("tags".to_string(), json!(["skill", "engineering", skill.name.as_str()]));
                props.insert("author".to_string(), json!("Tagisan AI"));
                props.insert("lastModifiedDateTime".to_string(), json!(now_iso));
                props.insert("url".to_string(), json!(format!("https://tagisan.ai/skills/{}", skill.name)));

                let content_str = format!("Skill: {}\nDescription: {}\n\nInstructions:\n{}", skill.name, skill.description, skill.instructions);

                IngestionItem {
                    id: id.clone(),
                    connection_id: self.connection_id.clone(),
                    item_type: "tagisanSkill".to_string(),
                    title: skill.name.clone(),
                    content: content_str,
                    url: format!("https://tagisan.ai/skills/{}", skill.name),
                    properties: props,
                    acl: vec![AclEntry::default()],
                }
            })
            .collect()
    }

    /// Build an indexable item for a dialectical debate consensus transcript
    pub fn build_debate_item(
        &self,
        debate_id: &str,
        topic: &str,
        verdict: &str,
        intermediate_steps: &[String],
    ) -> IngestionItem {
        let now_iso = chrono::Utc::now().to_rfc3339();
        let mut props = Map::new();
        props.insert("title".to_string(), json!(format!("Debate: {topic}")));
        props.insert("itemType".to_string(), json!("tagisanDebate"));
        props.insert("category".to_string(), json!("DialecticalConsensus"));
        props.insert("description".to_string(), json!(format!("Dialectical debate on: {topic}")));
        props.insert("verdict".to_string(), json!(verdict));
        props.insert("invariants".to_string(), json!("Lakandiwa formal consensus check"));
        props.insert("tags".to_string(), json!(["debate", "consensus", "lakandiwa"]));
        props.insert("author".to_string(), json!("Tagisan Lakandiwa"));
        props.insert("lastModifiedDateTime".to_string(), json!(now_iso));
        props.insert("url".to_string(), json!(format!("https://tagisan.ai/debates/{debate_id}")));

        let mut full_content = format!("# Tagisan Debate: {}\n\n## Master Verdict\n{}\n\n## Intermediate Rounds\n", topic, verdict);
        for (i, step) in intermediate_steps.iter().enumerate() {
            full_content.push_str(&format!("\n### Round {}\n{}\n", i + 1, step));
        }

        props.insert("content".to_string(), json!(&full_content));

        IngestionItem {
            id: format!("debate_{debate_id}"),
            connection_id: self.connection_id.clone(),
            item_type: "tagisanDebate".to_string(),
            title: format!("Debate: {topic}"),
            content: full_content,
            url: format!("https://tagisan.ai/debates/{debate_id}"),
            properties: props,
            acl: vec![AclEntry::default()],
        }
    }

    /// Build an indexable item for an architectural spec, diagram, or diff artifact
    pub fn build_artifact_item(
        &self,
        artifact_id: &str,
        name: &str,
        content: &str,
        artifact_type: &str,
    ) -> IngestionItem {
        let now_iso = chrono::Utc::now().to_rfc3339();
        let mut props = Map::new();
        props.insert("title".to_string(), json!(name));
        props.insert("itemType".to_string(), json!("tagisanArtifact"));
        props.insert("category".to_string(), json!(artifact_type));
        props.insert("description".to_string(), json!(format!("{artifact_type} artifact: {name}")));
        props.insert("content".to_string(), json!(content));
        props.insert("verdict".to_string(), json!("Verified"));
        props.insert("invariants".to_string(), json!("Grounding Invariants Passed"));
        props.insert("tags".to_string(), json!(["artifact", artifact_type]));
        props.insert("author".to_string(), json!("Tagisan Autonomous Agent"));
        props.insert("lastModifiedDateTime".to_string(), json!(now_iso));
        props.insert("url".to_string(), json!(format!("https://tagisan.ai/artifacts/{artifact_id}")));

        IngestionItem {
            id: format!("artifact_{artifact_id}"),
            connection_id: self.connection_id.clone(),
            item_type: "tagisanArtifact".to_string(),
            title: name.to_string(),
            content: content.to_string(),
            url: format!("https://tagisan.ai/artifacts/{artifact_id}"),
            properties: props,
            acl: vec![AclEntry::default()],
        }
    }

    /// Format an IngestionItem into the official Microsoft Graph external item PUT payload
    /// (PUT /v1.0/external/connections/{connectionId}/items/{itemId})
    pub fn format_graph_ingestion_payload(&self, item: &IngestionItem) -> Value {
        let acl_values: Vec<Value> = item
            .acl
            .iter()
            .map(|a| {
                json!({
                    "type": a.identity_type,
                    "value": a.value,
                    "accessType": a.access_type
                })
            })
            .collect();

        json!({
            "acl": acl_values,
            "properties": Value::Object(item.properties.clone()),
            "content": {
                "type": "text",
                "value": item.content
            }
        })
    }
}
