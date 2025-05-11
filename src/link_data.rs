use serde::Deserialize;
use std::collections::{HashMap, HashSet};

/// Represents the configuration for a single type of link field.
/// Loaded from `rstparser_links.toml`.
#[derive(Deserialize, Debug, Clone)]
pub struct LinkTypeConfig {
    pub name: String,
    // Placeholder for future enhancements, e.g.:
    // pub custom_backlink_suffix: Option<String>,
    // pub presentation_hint: Option<String>,
}

/// Represents the overall link configuration loaded from the TOML file.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct LinkConfig {
    #[serde(rename = "links", default)]
    pub link_types: Vec<LinkTypeConfig>,
}

/// Data stored for each directive in the LinkGraph.
/// Tracks both outgoing links (from this directive) and incoming links (to this directive).
#[derive(Debug, Clone, Default)]
pub struct LinkNodeData {
    /// Key: Original link field name (e.g., "derives", "tests").
    /// Value: List of target directive instance IDs.
    pub outgoing_links: HashMap<String, Vec<String>>,

    /// Key: Backlink field name (e.g., "derives_back", "tests_back").
    /// Value: List of source directive instance IDs that link to this directive via this backlink type.
    pub incoming_links: HashMap<String, Vec<String>>,
}

/// The LinkGraph stores relationships between directives.
/// Key: DirectiveInstanceId (String) of a directive.
/// Value: LinkNodeData for that directive.
pub type LinkGraph = HashMap<String, LinkNodeData>;

/// Loads link configuration from the specified TOML file path.
/// If the file does not exist, it returns a default (empty) LinkConfig.
/// Errors during reading or parsing will be propagated.
pub fn load_link_config(path: &str) -> Result<LinkConfig, Box<dyn std::error::Error>> {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            let config: LinkConfig = toml::from_str(&contents)?;
            Ok(config)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(LinkConfig::default()) // Return default config if file not found
        }
        Err(e) => Err(Box::new(e)), // Propagate other errors
    }
}

/// Removes all link information associated with the given `ids_to_remove`.
/// This involves:
/// 1. Removing these IDs from the `incoming_links` of any nodes they previously linked to.
/// 2. Removing the entries for `ids_to_remove` themselves from the graph.
pub fn remove_links_for_ids(graph: &mut LinkGraph, ids_to_remove: &HashSet<String>) {
    // Phase 1: Collect information about which incoming links to update.
    // Store as (target_id, backlink_field_name, id_of_source_to_remove_from_target's_incoming_list)
    let mut incoming_link_updates_to_make: Vec<(String, String, String)> = Vec::new();

    for removed_id in ids_to_remove {
        if let Some(removed_node_data) = graph.get(removed_id) {
            // For each outgoing link from the node being removed,
            // find the target and record that this `removed_id` should be
            // removed from the target's incoming links.
            for (link_field_name, target_ids) in &removed_node_data.outgoing_links {
                let backlink_field_name = format!("{}_back", link_field_name);
                for target_id in target_ids {
                    // Only update if the target itself is not being removed in this batch.
                    if !ids_to_remove.contains(target_id) {
                        incoming_link_updates_to_make.push((
                            target_id.clone(),
                            backlink_field_name.clone(),
                            removed_id.clone(),
                        ));
                    }
                }
            }
        }
    }

    // Perform the updates to incoming links.
    for (target_id, backlink_field_name, source_to_remove) in incoming_link_updates_to_make {
        if let Some(target_node_data) = graph.get_mut(&target_id) {
            if let Some(incoming_sources) = target_node_data.incoming_links.get_mut(&backlink_field_name) {
                incoming_sources.retain(|id| *id != source_to_remove);
                if incoming_sources.is_empty() {
                    target_node_data.incoming_links.remove(&backlink_field_name);
                }
            }
        }
    }

    // Phase 2: Remove the specified directive nodes themselves from the graph.
    // This handles removing their outgoing_links and any incoming_links pointing to them
    // from other nodes that might also be in ids_to_remove.
    for id_to_remove in ids_to_remove {
        graph.remove(id_to_remove);
    }
}
