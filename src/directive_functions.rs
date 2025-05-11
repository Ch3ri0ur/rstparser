use crate::aggregator::DirectiveWithSource;
use crate::link_data::{LinkConfig, LinkGraph};
use std::collections::HashMap; // Removed HashSet
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// Type alias for the main directive storage, to be passed to functions.
pub type AllDirectivesMap = HashMap<PathBuf, HashMap<String, Arc<Mutex<DirectiveWithSource>>>>;

/// Trait for functions that can be applied to directives.
pub trait DirectiveFunction: Send + Sync {
    fn name(&self) -> &str;

    /// Applies the function's logic.
    fn apply(
        &self,
        directive_id: &str,
        directive_data: &DirectiveWithSource,
        all_directives_map: &AllDirectivesMap,
        link_graph: &mut LinkGraph,
        link_config: &LinkConfig,
    ) -> Result<(), String>;
}

/// Function to process backlinks.
pub struct BacklinkFunction;

impl DirectiveFunction for BacklinkFunction {
    fn name(&self) -> &str {
        "BacklinkFunction"
    }

    fn apply(
        &self,
        directive_id: &str,
        directive_data: &DirectiveWithSource,
        _all_directives_map: &AllDirectivesMap, // Not directly used for now
        link_graph: &mut LinkGraph,
        link_config: &LinkConfig,
    ) -> Result<(), String> {
        let directive_options = &directive_data.directive.options;
        // Stores (field_name_of_link, source_directive_id, Vec<target_directive_ids>)
        let mut links_to_process: Vec<(String, String, Vec<String>)> = Vec::new();

        // --- Pass 1: Collect all link information and ensure all involved nodes exist ---
        for link_type_cfg in &link_config.link_types {
            if let Some(target_ids_str) = directive_options.get(&link_type_cfg.name) {
                if target_ids_str.is_empty() {
                    continue;
                }
                let current_target_ids: Vec<String> = target_ids_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if !current_target_ids.is_empty() {
                    // Add to list for processing in Pass 3
                    links_to_process.push((
                        link_type_cfg.name.clone(),
                        directive_id.to_string(),
                        current_target_ids.clone(),
                    ));
                    
                    // Ensure source node exists
                    link_graph.entry(directive_id.to_string()).or_default();
                    // Ensure all target nodes exist
                    for target_id in &current_target_ids {
                        if *target_id != directive_id { // Avoid redundant self-entry if already done by source
                            link_graph.entry(target_id.clone()).or_default();
                        }
                    }
                }
            }
        }

        // --- Pass 2: Clear old outgoing links for the current source directive ---
        // This is done after ensuring the source node exists from Pass 1.
        if let Some(source_node_data) = link_graph.get_mut(directive_id) {
            source_node_data.outgoing_links.clear();
        } else {
            // This should not happen if Pass 1 worked, but as a safeguard:
            eprintln!("Error: Source node '{}' not found in link_graph for clearing outgoing links. Inconsistency.", directive_id);
            // If it doesn't exist, there's nothing to clear, but it implies an issue.
        }
        
        // --- Pass 3: Process collected links to update graph edges (outgoing and incoming) ---
        for (field_name, source_id_str, target_ids_vec) in links_to_process {
            // Update outgoing links for the source_id_str
            // source_id_str here is always the current directive_id
            if let Some(source_node_data) = link_graph.get_mut(&source_id_str) {
                source_node_data
                    .outgoing_links
                    .entry(field_name.clone()) // field_name is the original link field, e.g., "derives"
                    .or_default()
                    .extend(target_ids_vec.iter().cloned());
            }

            // Update incoming links for each target_id in target_ids_vec
            for target_id in target_ids_vec {
                if target_id == source_id_str { 
                    eprintln!("Warning: Directive '{}' in file '{}' has a self-referential link in field '{}'.", source_id_str, directive_data.source_file, field_name);
                    continue;
                }
                if let Some(target_node_data) = link_graph.get_mut(&target_id) {
                    let backlink_field_name = format!("{}_back", field_name);
                    let incoming_for_field = target_node_data.incoming_links.entry(backlink_field_name).or_default();
                    if !incoming_for_field.contains(&source_id_str) {
                        incoming_for_field.push(source_id_str.clone());
                    }
                } else {
                    // This should ideally not be reached if Pass 1 correctly ensures all nodes exist.
                    eprintln!("Error: Target node '{}' not found in link_graph when trying to add incoming link from '{}' (field: {}). Inconsistency.", target_id, source_id_str, field_name);
                }
            }
        }
        Ok(())
    }
}

pub struct FunctionApplicator {
    functions: Vec<Box<dyn DirectiveFunction>>,
    link_config: Arc<LinkConfig>,
}

impl FunctionApplicator {
    pub fn new(link_config: Arc<LinkConfig>) -> Self {
        let mut functions: Vec<Box<dyn DirectiveFunction>> = Vec::new();
        functions.push(Box::new(BacklinkFunction));
        Self { functions, link_config }
    }

    pub fn apply_to_directive(
        &self,
        directive_id: &str,
        directive_data: &DirectiveWithSource,
        all_directives_map: &AllDirectivesMap,
        link_graph: &mut LinkGraph,
    ) {
        for function in &self.functions {
            if let Err(e) = function.apply(
                directive_id,
                directive_data,
                all_directives_map,
                link_graph,
                &self.link_config,
            ) {
                eprintln!(
                    "Error applying function '{}' to directive '{}': {}",
                    function.name(),
                    directive_id,
                    e
                );
            }
        }
    }

    pub fn apply_to_all(
        &self,
        current_directives_map: &AllDirectivesMap,
        link_graph: &mut LinkGraph,
    ) {
        // Clear all incoming links before full reprocessing.
        // Outgoing links are cleared per-directive within BacklinkFunction::apply (Pass 2).
        for node_data in link_graph.values_mut() {
            node_data.incoming_links.clear();
        }

        // It's also important to remove LinkGraph nodes for directives that no longer exist.
        let mut valid_directive_ids = std::collections::HashSet::new();
        for file_directives in current_directives_map.values() {
            for id in file_directives.keys() {
                valid_directive_ids.insert(id.clone());
            }
        }
        link_graph.retain(|id, _| valid_directive_ids.contains(id));


        for file_directives in current_directives_map.values() {
            for (id, directive_arc) in file_directives.iter() {
                let directive_data_guard = directive_arc.lock().unwrap();
                // Ensure node for current directive exists before applying (important if it has no outgoing links but might get incoming)
                // This is now handled in Pass 1 of BacklinkFunction::apply
                // link_graph.entry(id.clone()).or_default(); 
                self.apply_to_directive(id, &directive_data_guard, current_directives_map, link_graph);
            }
        }
    }

    /// Applies all registered functions to a specific subset of directives.
    /// This is intended for incremental updates where only some directives need reprocessing.
    /// It assumes that any necessary cleanup of old links related to these directives
    /// (e.g., using `link_data::remove_links_for_ids`) has been done beforehand if these
    /// directives are being re-evaluated.
    pub fn apply_to_subset(
        &self,
        directives_to_process: &[Arc<Mutex<DirectiveWithSource>>],
        all_directives_map: &AllDirectivesMap, // Full map for contextual lookups by functions
        link_graph: &mut LinkGraph,
    ) {
        for directive_arc in directives_to_process {
            let directive_data_guard = directive_arc.lock().unwrap();
            // apply_to_directive will call each function's apply method.
            // For BacklinkFunction, its apply method will:
            // 1. Ensure the node for directive_data_guard.id exists.
            // 2. Clear its old outgoing links.
            // 3. Rebuild its outgoing links and update incoming links on its targets.
            self.apply_to_directive(
                &directive_data_guard.id,
                &directive_data_guard,
                all_directives_map,
                link_graph,
            );
        }
    }
}
