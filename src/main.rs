mod parser;
mod file_walker;
mod aggregator;
mod processor;
mod extractor;

use std::collections::HashMap;
use std::path::PathBuf;
use std::process;
use std::sync::{Arc, Mutex}; // Added Arc, Mutex
use clap::{Parser, ValueEnum};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, event::EventKind};
use std::sync::mpsc::channel;
use file_walker::FileWalker;
use processor::Processor;
use aggregator::{Aggregator, GroupBy, DirectiveWithSource}; // Added DirectiveWithSource

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
    /// Group by directive name (one JSON file per directive type)
    DirectiveName,
    /// Group all directives into a single JSON file
    All,
    /// Group by source file (one JSON file per source file)
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
    // Parse command line arguments
    let cli = Cli::parse();

    if cli.watch {
        println!("Watch mode enabled. Watching directory: {}. Press Ctrl+C to exit.", &cli.dir);

        // Create a channel to receive events.
        let (tx, rx) = channel();

        // Create a file watcher.
        let mut watcher = match RecommendedWatcher::new(tx, notify::Config::default()) {
            Ok(watcher) => watcher,
            Err(e) => {
                eprintln!("Error creating file watcher: {}", e);
                process::exit(1);
            }
        };

        // Add the path to be watched.
        if let Err(e) = watcher.watch(PathBuf::from(&cli.dir).as_path(), RecursiveMode::Recursive) {
            eprintln!("Error watching path {}: {}", &cli.dir, e);
            process::exit(1);
        }

        // --- Initial Scan Logic ---
        let extensions: Vec<String> = cli.extensions
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        let directives_to_find: Vec<String> = cli.directives
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        if directives_to_find.is_empty() {
            eprintln!("Error: At least one directive name must be specified for watching.");
            process::exit(1);
        }
        
        let output_dir = PathBuf::from(&cli.output);
        
        // Ensure output directory exists for initial write
        if !output_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&output_dir) {
                eprintln!("Error creating output directory {}: {}", output_dir.display(), e);
                process::exit(1);
            }
        }

        let walker = if let Some(depth) = cli.max_depth {
            FileWalker::new()
                .with_extensions(extensions.clone()) // Clone for watcher's potential re-use
                .with_max_depth(depth)
        } else {
            FileWalker::new()
                .with_extensions(extensions.clone()) // Clone for watcher's potential re-use
        };
        
        println!("Performing initial scan of '{}'...", &cli.dir);
        let initial_files = match walker.find_files(&cli.dir) {
            Ok(files) => files,
            Err(err) => {
                eprintln!("Error during initial file scan: {}", err);
                process::exit(1);
            }
        };
        
        println!("Initial scan found {} files to process.", initial_files.len());
        
        let processor = Processor::new(directives_to_find.clone()); // Clone for watcher

        // --- Modified current_directives_with_source structure and initialization ---
        let mut initial_processed_directives_map: HashMap<PathBuf, HashMap<String, DirectiveWithSource>> = HashMap::new();
        match processor.process_files(initial_files) {
            Ok(directives_vec) => {
                for dws in directives_vec {
                    let file_path_buf = PathBuf::from(&dws.source_file);
                    let directive_id = dws.directive.options.get("id")
                        .map(|id_val| id_val.clone())
                        .unwrap_or_else(|| {
                            format!("{}:{}:{}",
                                file_path_buf.display(),
                                dws.directive.name,
                                dws.line_number.unwrap_or(0) // Should always have line number from parser
                            )
                        });
                    initial_processed_directives_map
                        .entry(file_path_buf)
                        .or_default()
                        .insert(directive_id, dws);
                }
            }
            Err(err) => {
                eprintln!("Error processing files during initial scan: {}", err);
                process::exit(1);
            }
        }
        
        let current_directives_with_source = Arc::new(Mutex::new(initial_processed_directives_map));
        // --- End of modification ---
        
        // Count total directives for initial scan log
        let initial_directive_count = current_directives_with_source.lock().unwrap().values().map(|fm| fm.len()).sum::<usize>();
        println!("Initial scan found {} directives.", initial_directive_count);
        
        let aggregator = Aggregator::new(output_dir.clone(), cli.group_by.into());
        match aggregator.aggregate_to_json_from_map(current_directives_with_source.clone()) { // Pass Arc<Mutex<HashMap<...>>>
            Ok(output_files) => {
                println!("Initial aggregation complete. Wrote {} JSON files:", output_files.len());
                for file in output_files {
                    println!("  {}", file.display());
                }
            },
            Err(err) => {
                eprintln!("Error writing JSON files during initial aggregation: {}", err);
                process::exit(1);
            }
        }
        // --- End of Initial Scan Logic ---

        // Event loop.
        // Event loop.
        loop {
            match rx.recv() {
                Ok(event_result) => {
                    match event_result {
                        Ok(event) => {
                            println!("File event: {:?}", event);
                            let mut changed = false;
                            
                            let relevant_event_paths: Vec<PathBuf> = if !event.kind.is_remove() {
                                event.paths.iter().filter(|p| {
                                    extensions.iter().any(|ext| {
                                        p.extension().map_or(false, |file_ext| file_ext == ext.trim_start_matches('.'))
                                    })
                                }).cloned().collect()
                            } else {
                                event.paths.clone() // For remove, take all paths
                            };

                            // Skip if no relevant files for create/modify (remove events might have empty relevant_event_paths if a dir is removed, but logic handles it)
                            if !event.kind.is_remove() && relevant_event_paths.is_empty() {
                                continue;
                            }

                            let mut global_directives_map = current_directives_with_source.lock().unwrap();

                            match event.kind {
                                EventKind::Create(_) | EventKind::Modify(_) => {
                                    // This block should only execute if relevant_event_paths is not empty for Create/Modify
                                    if event.kind.is_create() {
                                        println!("File(s) created: {:?}", relevant_event_paths);
                                    } else {
                                        println!("File(s) modified: {:?}", relevant_event_paths);
                                    }
                                    
                                    for path_to_process in &relevant_event_paths { // Iterate over ref
                                        match processor.process_file(path_to_process) { // process_file for single path
                                            Ok(processed_directives_vec) => {
                                                let mut file_specific_map: HashMap<String, DirectiveWithSource> = HashMap::new();
                                                for dws in processed_directives_vec {
                                                    let directive_id = dws.directive.options.get("id")
                                                        .map(|id_val| id_val.clone())
                                                        .unwrap_or_else(|| {
                                                            format!("{}:{}:{}",
                                                                path_to_process.display(), // Use the actual path being processed
                                                                dws.directive.name,
                                                                dws.line_number.unwrap_or(0)
                                                            )
                                                        });
                                                    file_specific_map.insert(directive_id, dws);
                                                }
                                                global_directives_map.insert(path_to_process.clone(), file_specific_map);
                                                changed = true;
                                                println!("  Updated/added directives for {}", path_to_process.display());
                                            }
                                            Err(e) => eprintln!("  Error processing file {}: {}", path_to_process.display(), e),
                                        }
                                    }
                                }
                                EventKind::Remove(_) => {
                                    println!("Path(s) removed: {:?}", relevant_event_paths);
                                    for removed_path_item in relevant_event_paths {
                                        // If it's a file, remove it directly.
                                        // If it's a directory, iterate and remove all files within it from our map.
                                        let mut keys_to_remove: Vec<PathBuf> = Vec::new();
                                        if removed_path_item.is_dir() {
                                            for (key, _value) in global_directives_map.iter() {
                                                if key.starts_with(&removed_path_item) {
                                                    keys_to_remove.push(key.clone());
                                                }
                                            }
                                        } else {
                                            keys_to_remove.push(removed_path_item.clone());
                                        }

                                        for key_to_remove in keys_to_remove {
                                            if global_directives_map.remove(&key_to_remove).is_some() {
                                                println!("  Removed directives from cache for {}", key_to_remove.display());
                                                changed = true;
                                            }
                                        }
                                    }
                                }
                                _ => { /* Other events ignored */ }
                            }
                            // Drop the lock before aggregation
                            drop(global_directives_map);

                            if changed {
                                // Re-acquire lock for reading total count, or pass a clone if aggregator needs it locked.
                                // For simplicity, let's re-acquire for count and aggregator will clone the Arc.
                                let final_directive_count = current_directives_with_source.lock().unwrap().values().map(|fm| fm.len()).sum::<usize>();
                                println!("Re-aggregating {} total directives...", final_directive_count);
                                match aggregator.aggregate_to_json_from_map(current_directives_with_source.clone()) {
                                    Ok(output_files) => {
                                        println!("Aggregation complete. Wrote {} JSON files:", output_files.len());
                                        for file in output_files {
                                            println!("  {}", file.display());
                                        }
                                    },
                                    Err(err) => {
                                        eprintln!("Error writing JSON files after event: {}", err);
                                    }
                                }
                            }
                        }
                        Err(e) => eprintln!("Watch error: {:?}", e),
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving event: {}", e);
                    break;
                }
            }
        }
    } else {
        // Existing logic for non-watch mode
        let extensions: Vec<String> = cli.extensions
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        let directives: Vec<String> = cli.directives
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        if directives.is_empty() {
            eprintln!("Error: At least one directive name must be specified");
            process::exit(1);
        }
        
        let output_dir = PathBuf::from(&cli.output);
        
        let walker = if let Some(depth) = cli.max_depth {
            FileWalker::new()
                .with_extensions(extensions)
                .with_max_depth(depth)
        } else {
            FileWalker::new()
                .with_extensions(extensions)
        };
        
        let files = match walker.find_files(&cli.dir) {
            Ok(files) => files,
            Err(err) => {
                eprintln!("Error finding files: {}", err);
                process::exit(1);
            }
        };
        
        println!("Found {} files to process", files.len());
        
        let processor = Processor::new(directives);
        // --- Non-watch mode: Convert Vec<DirectiveWithSource> to the new map structure before aggregation ---
        let directives_vec = match processor.process_files(files) {
            Ok(directives) => directives,
            Err(err) => {
                eprintln!("Error processing files: {}", err);
                process::exit(1);
            }
        };
        
        let mut directives_map_for_aggregator: HashMap<PathBuf, HashMap<String, DirectiveWithSource>> = HashMap::new();
        for dws in directives_vec {
            let file_path_buf = PathBuf::from(&dws.source_file);
            let directive_id = dws.directive.options.get("id")
                .map(|id_val| id_val.clone())
                .unwrap_or_else(|| {
                    format!("{}:{}:{}",
                        file_path_buf.display(),
                        dws.directive.name,
                        dws.line_number.unwrap_or(0)
                    )
                });
            directives_map_for_aggregator
                .entry(file_path_buf)
                .or_default()
                .insert(directive_id, dws);
        }
        // --- End of non-watch mode adaptation ---
        
        let total_directives_found = directives_map_for_aggregator.values().map(|fm| fm.len()).sum::<usize>();
        println!("Found {} directives", total_directives_found);
        
        let aggregator = Aggregator::new(output_dir, cli.group_by.into());
        // For non-watch mode, we pass the owned map.
        // The aggregator will need a new method or an adapter if we want to keep aggregate_to_json_from_map for Arc<Mutex<>>
        // For now, let's assume we'll adapt aggregator or create a new path.
        // For simplicity in this step, let's imagine aggregate_to_json can be overloaded or a new one is called.
        // We'll create a new method in aggregator `aggregate_map_to_json` for owned map.
        match aggregator.aggregate_map_to_json(directives_map_for_aggregator) {
            Ok(output_files) => {
                println!("Successfully wrote {} JSON files:", output_files.len());
                for file in output_files {
                    println!("  {}", file.display());
                }
            },
            Err(err) => {
                eprintln!("Error writing JSON files: {}", err);
                process::exit(1);
            }
        }
    }
}
