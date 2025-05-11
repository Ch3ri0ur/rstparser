mod parser;
mod file_walker;
mod aggregator;
mod processor;
mod extractor;
mod link_data;
mod directive_functions;

// rstparser crate's own modules (if main.rs is treated as part of the crate)
// If main.rs is a binary using rstparser as a library, these would be:
// use rstparser::file_walker; etc.
// For now, assuming main.rs can access sibling modules directly or via `crate::`
use crate::file_walker::FileWalker;
use crate::processor::Processor;
use crate::aggregator::{Aggregator, GroupBy, DirectiveWithSource};
use crate::link_data::{load_link_config, LinkConfig, LinkGraph, remove_links_for_ids}; // Added remove_links_for_ids
use crate::directive_functions::FunctionApplicator; // Added

use std::collections::{HashMap, HashSet}; // Added HashSet
use std::path::PathBuf;
use std::process;
use std::sync::{Arc, Mutex};
use clap::{Parser, ValueEnum};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, event::EventKind};
use std::sync::mpsc::channel;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Directory to search for RST files
    #[arg(short, long, default_value = ".")]
    dir: String,

    /// File extensions to search (comma-separated)
    #[arg(short, long, default_value = "rst,py,cpp")]
    extensions: String,

    /// Directive names to search for (comma-separated)
    #[arg(short = 'D', long)]
    directives: String,

    /// Output directory for JSON files
    #[arg(short, long, default_value = "output")]
    output: String,

    /// How to group directives in output files
    #[arg(short, long, value_enum, default_value_t = GroupByArg::DirectiveName)]
    group_by: GroupByArg,

    /// Maximum directory depth to search
    #[arg(short, long)]
    max_depth: Option<usize>,

    /// Enable file watching mode
    #[arg(short, long, default_value_t = false)]
    watch: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum GroupByArg {
    DirectiveName,
    All,
    SourceFile,
}

impl From<GroupByArg> for GroupBy {
    fn from(arg: GroupByArg) -> Self {
        match arg {
            GroupByArg::DirectiveName => GroupBy::DirectiveName,
            GroupByArg::All => GroupBy::All,
            GroupByArg::SourceFile => GroupBy::SourceFile,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let link_config_path = "rstparser_links.toml";
    let link_config = match load_link_config(link_config_path) {
        Ok(cfg) => {
            println!("Successfully loaded link configuration from '{}'. Found {} link types.", link_config_path, cfg.link_types.len());
            Arc::new(cfg)
        }
        Err(e) => {
            eprintln!("Warning: Could not load link configuration from '{}': {}. Proceeding without link processing.", link_config_path, e);
            Arc::new(LinkConfig::default())
        }
    };

    let function_applicator = FunctionApplicator::new(link_config.clone());

    let extensions: Vec<String> = cli.extensions.split(',').map(|s| s.trim().to_string()).collect();
    let directives_to_find: Vec<String> = cli.directives.split(',').map(|s| s.trim().to_string()).collect();

    if directives_to_find.is_empty() {
        eprintln!("Error: At least one directive name must be specified.");
        process::exit(1);
    }

    let output_dir = PathBuf::from(&cli.output);
    if !output_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&output_dir) {
            eprintln!("Error creating output directory {}: {}", output_dir.display(), e);
            process::exit(1);
        }
    }
    
    let walker = if let Some(depth) = cli.max_depth {
        FileWalker::new().with_extensions(extensions.clone()).with_max_depth(depth)
    } else {
        FileWalker::new().with_extensions(extensions.clone())
    };

    let processor = Processor::new(directives_to_find.clone());
    let aggregator = Aggregator::new(output_dir.clone(), cli.group_by.into());


    if cli.watch {
        println!("Watch mode enabled. Watching directory: {}. Press Ctrl+C to exit.", &cli.dir);
        let (tx, rx) = channel();
        let mut watcher = match RecommendedWatcher::new(tx, notify::Config::default()) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Error creating file watcher: {}", e);
                process::exit(1);
            }
        };
        if let Err(e) = watcher.watch(PathBuf::from(&cli.dir).as_path(), RecursiveMode::Recursive) {
            eprintln!("Error watching path {}: {}", &cli.dir, e);
            process::exit(1);
        }

        // --- Initial Scan Logic for Watch Mode ---
        println!("Performing initial scan of '{}'...", &cli.dir);
        let initial_files = match walker.find_files(&cli.dir) {
            Ok(files) => files,
            Err(err) => {
                eprintln!("Error during initial file scan: {}", err);
                process::exit(1);
            }
        };
        println!("Initial scan found {} files to process.", initial_files.len());

        let mut initial_processed_directives_map: HashMap<PathBuf, HashMap<String, Arc<Mutex<DirectiveWithSource>>>> = HashMap::new();
        match processor.process_files_watch(initial_files) { // Assuming process_files_watch returns Vec<Arc<Mutex<Dws>>> per file or similar
            Ok(processed_map_from_processor) => { // This needs to align with Processor's output for watch mode
                for (file_path, directives_in_file_vec) in processed_map_from_processor {
                     let canonical_file_path = match std::fs::canonicalize(&file_path) {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Warning: Failed to canonicalize path during initial scan {}: {}", file_path.display(), e);
                            file_path // Fallback
                        }
                    };
                    let mut file_map = HashMap::new();
                    for dws_arc in directives_in_file_vec {
                        let dws_guard = dws_arc.lock().unwrap();
                        file_map.insert(dws_guard.id.clone(), dws_arc.clone());
                    }
                    initial_processed_directives_map.insert(canonical_file_path, file_map);
                }
            }
            Err(err) => {
                eprintln!("Error processing files during initial scan: {}", err);
                process::exit(1);
            }
        }
        
        let current_directives_with_source = Arc::new(Mutex::new(initial_processed_directives_map));
        
        // --- Apply directive functions (Initial Scan for Watch Mode) ---
        let mut link_graph_watch = LinkGraph::default();
        println!("Applying directive functions (initial scan)...");
        let directives_map_guard = current_directives_with_source.lock().unwrap();
        function_applicator.apply_to_all(&directives_map_guard, &mut link_graph_watch);
        drop(directives_map_guard); // Release lock
        println!("Directive functions applied. Link graph has {} entries.", link_graph_watch.len());
        let link_graph_arc_watch = Arc::new(Mutex::new(link_graph_watch));
        // --- End of applying directive functions ---

        let initial_directive_count = current_directives_with_source.lock().unwrap().values().map(|fm| fm.len()).sum::<usize>();
        println!("Initial scan found {} directives.", initial_directive_count);
        
        match aggregator.aggregate_to_json_from_map_with_links(current_directives_with_source.clone(), link_graph_arc_watch.clone()) {
            Ok(output_files) => {
                println!("Initial aggregation complete. Wrote {} JSON files:", output_files.len());
                for file in output_files { println!("  {}", file.display()); }
            },
            Err(err) => {
                eprintln!("Error writing JSON files during initial aggregation: {}", err);
                process::exit(1);
            }
        }

        // Event loop for watch mode
        loop {
            match rx.recv() {
                Ok(event_result) => match event_result {
                    Ok(event) => {
                        println!("File event: {:?}", event);
                        let mut changed_anything_globally = false;
                        let relevant_event_paths: Vec<PathBuf> = event.paths.iter().filter(|p| {
                            !event.kind.is_remove() && extensions.iter().any(|ext| p.extension().map_or(false, |file_ext| file_ext == ext.trim_start_matches('.')))
                        }).cloned().collect();
                        
                        let mut global_directives_map_guard = current_directives_with_source.lock().unwrap();
                        let mut link_graph_guard = link_graph_arc_watch.lock().unwrap();
                        
                        let mut ids_to_clear_from_graph = HashSet::new(); // IDs whose links need to be removed before reprocessing
                        let mut arcs_for_subset_application: Vec<Arc<Mutex<DirectiveWithSource>>> = Vec::new();
                        let mut affected_ids_for_neighbor_scan = HashSet::new(); // IDs that were modified or removed, to find their neighbors

                        match event.kind {
                            EventKind::Create(_) | EventKind::Modify(_) => {
                                if relevant_event_paths.is_empty() { continue; }
                                println!("File(s) created/modified: {:?}", relevant_event_paths);
                                for path_to_process_orig in &relevant_event_paths {
                                    let canonical_path = match std::fs::canonicalize(path_to_process_orig) {
                                        Ok(p) => p,
                                        Err(e) => {
                                            eprintln!("Warning: Failed to canonicalize path for event {}: {}", path_to_process_orig.display(), e);
                                            path_to_process_orig.clone()
                                        }
                                    };

                                    // Collect old IDs from this file to clear their links and find neighbors
                                    if let Some(old_file_directives) = global_directives_map_guard.get(&canonical_path) {
                                        for old_id in old_file_directives.keys() {
                                            ids_to_clear_from_graph.insert(old_id.clone());
                                            affected_ids_for_neighbor_scan.insert(old_id.clone());
                                        }
                                    }
                                    
                                    match processor.process_file_watch(&canonical_path) {
                                        Ok(processed_directives_arcs_for_file) => {
                                            let mut new_file_map = HashMap::new();
                                            for dws_arc in processed_directives_arcs_for_file {
                                                let dws_guard = dws_arc.lock().unwrap();
                                                new_file_map.insert(dws_guard.id.clone(), dws_arc.clone());
                                                arcs_for_subset_application.push(dws_arc.clone()); 
                                                ids_to_clear_from_graph.insert(dws_guard.id.clone()); // Also clear new IDs in case they existed before with different content
                                                affected_ids_for_neighbor_scan.insert(dws_guard.id.clone());
                                            }
                                            global_directives_map_guard.insert(canonical_path.clone(), new_file_map);
                                            changed_anything_globally = true;
                                            println!("  Updated/added directives for {}", canonical_path.display());
                                        }
                                        Err(e) => eprintln!("  Error processing file {}: {}", canonical_path.display(), e),
                                    }
                                }
                            }
                            EventKind::Remove(_) => {
                                println!("Path(s) removed: {:?}", event.paths);
                                for removed_path_item_orig in &event.paths {
                                    let path_key_candidate = match std::fs::canonicalize(removed_path_item_orig) {
                                        Ok(p) => p,
                                        Err(_) => removed_path_item_orig.clone(), 
                                    };
                                    
                                    let keys_to_remove_from_map: Vec<PathBuf> = global_directives_map_guard.keys()
                                        .filter(|k| **k == path_key_candidate || k.starts_with(&path_key_candidate))
                                        .cloned()
                                        .collect();
                                    
                                    for key_to_remove in keys_to_remove_from_map {
                                        if let Some(removed_file_directives) = global_directives_map_guard.remove(&key_to_remove) {
                                            for id in removed_file_directives.keys() {
                                                ids_to_clear_from_graph.insert(id.clone());
                                                affected_ids_for_neighbor_scan.insert(id.clone());
                                            }
                                            println!("  Removed directives from cache for {}", key_to_remove.display());
                                            changed_anything_globally = true;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }

                        if changed_anything_globally {
                            // Find neighbors of affected IDs (those that linked TO or were targeted BY affected_ids_for_neighbor_scan)
                            // This scan must happen BEFORE clearing links from the graph.
                            let mut neighbor_arcs_to_reprocess: HashMap<String, Arc<Mutex<DirectiveWithSource>>> = HashMap::new();
                            if !affected_ids_for_neighbor_scan.is_empty() {
                                println!("Scanning for neighbors of {} affected/removed IDs...", affected_ids_for_neighbor_scan.len());
                                for (source_id, node_data) in link_graph_guard.iter() {
                                    // Check if this source_id is one of the directly affected ones (already in arcs_for_subset_application or to be removed)
                                    // If not, check its links.
                                    if !affected_ids_for_neighbor_scan.contains(source_id) {
                                        for targets in node_data.outgoing_links.values() {
                                            if targets.iter().any(|target_id| affected_ids_for_neighbor_scan.contains(target_id)) {
                                                // This source_id links to an affected ID. It needs reprocessing.
                                                // Find its Arc<Mutex<Dws>> from global_directives_map_guard
                                                for file_map in global_directives_map_guard.values() {
                                                    if let Some(arc) = file_map.get(source_id) {
                                                        neighbor_arcs_to_reprocess.insert(source_id.clone(), arc.clone());
                                                        break;
                                                    }
                                                }
                                                break; // Found a reason to reprocess this source_id, move to next in graph
                                            }
                                        }
                                    }
                                }
                                // Also, directives that were targets of affected_ids_for_neighbor_scan might need reprocessing
                                // if their incoming links are their only reason for being in the graph or having certain data.
                                // However, apply_to_subset on the sources should update their incoming links.
                                // The main concern is if a neighbor's *only* connection was to a now-deleted/changed node.
                                // The `remove_links_for_ids` and subsequent `apply_to_subset` should handle this.
                            }
                            
                            // Add collected neighbors to the main list for subset application, avoiding duplicates
                            for (id, arc) in neighbor_arcs_to_reprocess {
                                if !arcs_for_subset_application.iter().any(|a| a.lock().unwrap().id == id) {
                                    arcs_for_subset_application.push(arc);
                                }
                            }


                            if !ids_to_clear_from_graph.is_empty() {
                                println!("Clearing links for {} directive IDs from graph...", ids_to_clear_from_graph.len());
                                remove_links_for_ids(&mut link_graph_guard, &ids_to_clear_from_graph);
                            }

                            if !arcs_for_subset_application.is_empty() {
                                println!("Re-applying directive functions to {} directives (modified + neighbors)...", arcs_for_subset_application.len());
                                function_applicator.apply_to_subset(&arcs_for_subset_application, &global_directives_map_guard, &mut link_graph_guard);
                            }
                            
                            // Final cleanup: remove any LinkGraph nodes for directives that no longer exist in global_directives_map_guard
                            let mut still_valid_directive_ids = HashSet::new();
                            for file_directives in global_directives_map_guard.values() {
                                for id in file_directives.keys() {
                                    still_valid_directive_ids.insert(id.clone());
                                }
                            }
                            link_graph_guard.retain(|id, _| still_valid_directive_ids.contains(id));
                            println!("Directive functions updated. Link graph has {} entries.", link_graph_guard.len());
                        }
                        
                        drop(link_graph_guard); 
                        drop(global_directives_map_guard); // Release before aggregator

                        if changed_anything_globally {
                            let final_directive_count = current_directives_with_source.lock().unwrap().values().map(|fm| fm.len()).sum::<usize>();
                            println!("Re-aggregating {} total directives...", final_directive_count);
                            match aggregator.aggregate_to_json_from_map_with_links(current_directives_with_source.clone(), link_graph_arc_watch.clone()) {
                                Ok(output_files) => {
                                    println!("Aggregation complete. Wrote {} JSON files:", output_files.len());
                                    for file in output_files { println!("  {}", file.display()); }
                                },
                                Err(err) => eprintln!("Error writing JSON files after event: {}", err),
                            }
                        }
                    }
                    Err(e) => eprintln!("Watch error: {:?}", e),
                },
                Err(e) => {
                    eprintln!("Error receiving event: {}", e);
                    break; // Exit loop on channel receive error
                }
            }
        }

    } else { // Non-watch mode
        let files = match walker.find_files(&cli.dir) {
            Ok(f) => f,
            Err(err) => {
                eprintln!("Error finding files: {}", err);
                process::exit(1);
            }
        };
        println!("Found {} files to process", files.len());

        // In non-watch mode, Processor returns Vec<DirectiveWithSource>
        // We need to convert this to HashMap<PathBuf, HashMap<String, Arc<Mutex<DirectiveWithSource>>>>
        // for FunctionApplicator and the new aggregator method.
        let directives_vec = match processor.process_files(files) { // process_files returns Vec<Dws>
            Ok(directives) => directives,
            Err(err) => {
                eprintln!("Error processing files: {}", err);
                process::exit(1);
            }
        };
        
        let mut directives_map_for_processing: HashMap<PathBuf, HashMap<String, Arc<Mutex<DirectiveWithSource>>>> = HashMap::new();
        for dws_val in directives_vec { // dws_val is DirectiveWithSource, not Arc<Mutex<Dws>>
            let file_path_buf = PathBuf::from(&dws_val.source_file);
            // Canonicalize paths for consistency, though less critical in non-watch mode if IDs are stable
            let canonical_file_path = match std::fs::canonicalize(&file_path_buf) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Warning: Failed to canonicalize path in non-watch mode {}: {}", file_path_buf.display(), e);
                    file_path_buf 
                }
            };
            
            // Ensure dws_val.source_file is updated if canonicalized, and ID uses it
            let mut dws_mut = dws_val; // Make it mutable to update source_file
            dws_mut.source_file = canonical_file_path.to_string_lossy().into_owned();

            let directive_id = dws_mut.id.clone(); // ID should already be generated by Processor

            directives_map_for_processing
                .entry(canonical_file_path)
                .or_default()
                .insert(directive_id, Arc::new(Mutex::new(dws_mut)));
        }

        // --- Apply directive functions (Non-Watch Mode) ---
        let mut link_graph_non_watch = LinkGraph::default();
        println!("Applying directive functions...");
        function_applicator.apply_to_all(&directives_map_for_processing, &mut link_graph_non_watch);
        println!("Directive functions applied. Link graph has {} entries.", link_graph_non_watch.len());
        // --- End of applying directive functions ---

        let total_directives_found = directives_map_for_processing.values().map(|fm| fm.len()).sum::<usize>();
        println!("Found {} directives", total_directives_found);
        
        match aggregator.aggregate_map_to_json_with_links(&directives_map_for_processing, &link_graph_non_watch) {
            Ok(output_files) => {
                println!("Successfully wrote {} JSON files:", output_files.len());
                for file in output_files { println!("  {}", file.display()); }
            },
            Err(err) => {
                eprintln!("Error writing JSON files: {}", err);
                process::exit(1);
            }
        }
    }
}
