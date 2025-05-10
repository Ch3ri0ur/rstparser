use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::error::Error;
use serde::{Serialize, Deserialize};
use crate::parser::Directive; // This should be fine as parser is a sibling module
use crate::link_data::LinkGraph; // Using rstparser:: as per compiler hints
use std::sync::{Arc, Mutex};

/// A struct representing a directive with its source file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectiveWithSource {
    pub directive: Directive,
    pub source_file: String, // Should be canonical path
    pub line_number: Option<usize>, // Optional line number where the directive was found
    pub id: String, // Unique ID for this directive instance
}

/// A struct specifically for JSON output, potentially enriched with link data.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DirectiveOutput {
    // Fields from Directive
    name: String,
    arguments: String,
    options: HashMap<String, String>, // Will include original + backlinks
    content: String,
    // Fields from DirectiveWithSource
    source_file: String,
    line_number: Option<usize>,
    id: String,
}

impl From<&DirectiveWithSource> for DirectiveOutput {
    fn from(dws: &DirectiveWithSource) -> Self {
        DirectiveOutput {
            name: dws.directive.name.clone(),
            arguments: dws.directive.arguments.clone(),
            options: dws.directive.options.clone(), // Start with original options
            content: dws.directive.content.clone(),
            source_file: dws.source_file.clone(),
            line_number: dws.line_number,
            id: dws.id.clone(),
        }
    }
}


/// A struct to handle aggregation of directives into JSON files
pub struct Aggregator {
    output_dir: PathBuf,
    group_by: GroupBy,
}

/// Enum to specify how directives should be grouped in output files
#[derive(Debug, Clone, Copy)]
pub enum GroupBy {
    DirectiveName,
    All,
    SourceFile,
}

impl Aggregator {
    pub fn new<P: AsRef<Path>>(output_dir: P, group_by: GroupBy) -> Self {
        Aggregator {
            output_dir: output_dir.as_ref().to_path_buf(),
            group_by,
        }
    }

    fn create_directive_outputs(
        directives_map: &HashMap<PathBuf, HashMap<String, Arc<Mutex<DirectiveWithSource>>>>,
        link_graph: &LinkGraph,
    ) -> Vec<DirectiveOutput> {
        let mut output_directives: Vec<DirectiveOutput> = Vec::new();
        for file_map in directives_map.values() {
            for dws_arc in file_map.values() {
                let dws_guard = dws_arc.lock().unwrap();
                let mut output_item = DirectiveOutput::from(&*dws_guard); // Deref guard

                // Add backlinks to options
                if let Some(node_data) = link_graph.get(&dws_guard.id) {
                    for (backlink_field_name, source_ids) in &node_data.incoming_links {
                        if !source_ids.is_empty() {
                            output_item.options.insert(backlink_field_name.clone(), source_ids.join(","));
                        }
                    }
                }
                output_directives.push(output_item);
            }
        }
        output_directives
    }
    
    fn aggregate_outputs_to_json_internal(
        &self,
        output_directives: Vec<DirectiveOutput>,
    ) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        fs::create_dir_all(&self.output_dir)?;
        let mut output_files = Vec::new();

        match self.group_by {
            GroupBy::DirectiveName => {
                let mut grouped: HashMap<String, Vec<&DirectiveOutput>> = HashMap::new();
                for item_ref in &output_directives {
                    grouped.entry(item_ref.name.clone()).or_default().push(item_ref);
                }
                for (name, group) in grouped {
                    let file_path = self.output_dir.join(format!("{}.json", name));
                    fs::write(&file_path, serde_json::to_string_pretty(&group)?)?;
                    output_files.push(file_path);
                }
            }
            GroupBy::All => {
                let file_path = self.output_dir.join("all_directives.json");
                fs::write(&file_path, serde_json::to_string_pretty(&output_directives)?)?;
                output_files.push(file_path);
            }
            GroupBy::SourceFile => {
                let mut grouped: HashMap<String, Vec<&DirectiveOutput>> = HashMap::new();
                for item_ref in &output_directives {
                    grouped.entry(item_ref.source_file.clone()).or_default().push(item_ref);
                }
                for (source_file, group) in grouped {
                    let file_name = Path::new(&source_file).file_name().and_then(|n| n.to_str()).unwrap_or("unknown_source").to_string();
                    let file_path = self.output_dir.join(format!("{}.json", file_name));
                    fs::write(&file_path, serde_json::to_string_pretty(&group)?)?;
                    output_files.push(file_path);
                }
            }
        }
        Ok(output_files)
    }

    // --- New methods for aggregating WITH link graph ---
    pub fn aggregate_to_json_from_map_with_links(
        &self,
        directives_map_arc: Arc<Mutex<HashMap<PathBuf, HashMap<String, Arc<Mutex<DirectiveWithSource>>>>>>,
        link_graph_arc: Arc<Mutex<LinkGraph>>,
    ) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let directives_map_guard = directives_map_arc.lock().unwrap();
        let link_graph_guard = link_graph_arc.lock().unwrap();
        let output_directives = Self::create_directive_outputs(&directives_map_guard, &link_graph_guard);
        drop(directives_map_guard);
        drop(link_graph_guard);
        self.aggregate_outputs_to_json_internal(output_directives)
    }

    pub fn aggregate_map_to_json_with_links(
        &self,
        directives_map: &HashMap<PathBuf, HashMap<String, Arc<Mutex<DirectiveWithSource>>>>,
        link_graph: &LinkGraph,
    ) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let output_directives = Self::create_directive_outputs(directives_map, link_graph);
        self.aggregate_outputs_to_json_internal(output_directives)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;
    use crate::link_data::LinkNodeData; // Changed to rstparser::

    // Helper to create a simple DirectiveWithSource for tests
    fn new_dws(name: &str, file: &str, line: usize, id_val: &str, options_map: Option<HashMap<String, String>>) -> DirectiveWithSource {
        DirectiveWithSource {
            directive: Directive {
                name: name.to_string(),
                arguments: "".to_string(),
                options: options_map.unwrap_or_default(),
                content: format!("Content for {}", id_val),
            },
            source_file: file.to_string(),
            line_number: Some(line),
            id: id_val.to_string(),
        }
    }


    #[test]
    fn test_aggregate_by_directive_name() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path();
        
        let d1 = new_dws("directive1", "file1.rst", 10, "d1f1", None);
        let d2 = new_dws("directive2", "file2.rst", 20, "d2f2", None);
        let d3 = new_dws("directive1", "file3.rst", 30, "d1f3", None);
        
        let directives_with_source = vec![d1, d2, d3];
        let mut directives_map: HashMap<PathBuf, HashMap<String, Arc<Mutex<DirectiveWithSource>>>> = HashMap::new();
        for dws_val in directives_with_source {
            let file_path_buf = PathBuf::from(&dws_val.source_file);
            let directive_id = dws_val.id.clone();
            directives_map
                .entry(file_path_buf)
                .or_default()
                .insert(directive_id, Arc::new(Mutex::new(dws_val)));
        }
        let link_graph = LinkGraph::new();
        
        let aggregator = Aggregator::new(output_path, GroupBy::DirectiveName);
        let output_files = aggregator.aggregate_map_to_json_with_links(&directives_map, &link_graph).unwrap();
        
        assert_eq!(output_files.len(), 2);
        let directive1_file = output_path.join("directive1.json");
        let directive2_file = output_path.join("directive2.json");
        assert!(directive1_file.exists());
        assert!(directive2_file.exists());
        
        let directive1_content: Vec<DirectiveOutput> = 
            serde_json::from_str(&fs::read_to_string(directive1_file).unwrap()).unwrap();
        let directive2_content: Vec<DirectiveOutput> = 
            serde_json::from_str(&fs::read_to_string(directive2_file).unwrap()).unwrap();
        
        assert_eq!(directive1_content.len(), 2);
        assert_eq!(directive2_content.len(), 1);
    }
    
    #[test]
    fn test_aggregate_all() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path();
        let d1 = new_dws("directive1", "file1.rst", 10, "d1f1", None);
        let d2 = new_dws("directive2", "file2.rst", 20, "d2f2", None);
        let directives_with_source = vec![d1, d2];
        let mut directives_map: HashMap<PathBuf, HashMap<String, Arc<Mutex<DirectiveWithSource>>>> = HashMap::new();
        for dws_val in directives_with_source {
            let file_path_buf = PathBuf::from(&dws_val.source_file);
            let directive_id = dws_val.id.clone();
            directives_map
                .entry(file_path_buf)
                .or_default()
                .insert(directive_id, Arc::new(Mutex::new(dws_val)));
        }
        let link_graph = LinkGraph::new();
        
        let aggregator = Aggregator::new(output_path, GroupBy::All);
        let output_files = aggregator.aggregate_map_to_json_with_links(&directives_map, &link_graph).unwrap();
        
        assert_eq!(output_files.len(), 1);
        let all_directives_file = output_path.join("all_directives.json");
        assert!(all_directives_file.exists());
        let content: Vec<DirectiveOutput> = 
            serde_json::from_str(&fs::read_to_string(all_directives_file).unwrap()).unwrap();
        assert_eq!(content.len(), 2);
    }
    
    #[test]
    fn test_aggregate_by_source_file() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path();
        let d1 = new_dws("directive1", "file1.rst", 10, "d1f1", None);
        let d2 = new_dws("directive2", "file1.rst", 20, "d2f1", None);
        let d3 = new_dws("directive3", "file2.rst", 30, "d3f2", None);
        let directives_with_source = vec![d1, d2, d3];
        let mut directives_map: HashMap<PathBuf, HashMap<String, Arc<Mutex<DirectiveWithSource>>>> = HashMap::new();
        for dws_val in directives_with_source {
            let file_path_buf = PathBuf::from(&dws_val.source_file);
            let directive_id = dws_val.id.clone();
            directives_map
                .entry(file_path_buf)
                .or_default()
                .insert(directive_id, Arc::new(Mutex::new(dws_val)));
        }
        let link_graph = LinkGraph::new();

        let aggregator = Aggregator::new(output_path, GroupBy::SourceFile);
        let output_files = aggregator.aggregate_map_to_json_with_links(&directives_map, &link_graph).unwrap();
        
        assert_eq!(output_files.len(), 2);
        let file1_output = output_path.join("file1.rst.json");
        let file2_output = output_path.join("file2.rst.json");
        assert!(file1_output.exists());
        assert!(file2_output.exists());
        
        let file1_content: Vec<DirectiveOutput> = 
            serde_json::from_str(&fs::read_to_string(file1_output).unwrap()).unwrap();
        let file2_content: Vec<DirectiveOutput> = 
            serde_json::from_str(&fs::read_to_string(file2_output).unwrap()).unwrap();
        assert_eq!(file1_content.len(), 2);
        assert_eq!(file2_content.len(), 1);
    }

    #[test]
    fn test_aggregate_with_links() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path();

        let mut opts_d1 = HashMap::new();
        opts_d1.insert("links_to".to_string(), "d2".to_string());

        let d1_arc = Arc::new(Mutex::new(new_dws("directive1", "file1.rst", 10, "d1", Some(opts_d1))));
        let d2_arc = Arc::new(Mutex::new(new_dws("directive2", "file1.rst", 20, "d2", None)));

        let mut directives_map: HashMap<PathBuf, HashMap<String, Arc<Mutex<DirectiveWithSource>>>> = HashMap::new();
        let mut file1_map = HashMap::new();
        file1_map.insert("d1".to_string(), d1_arc.clone());
        file1_map.insert("d2".to_string(), d2_arc.clone());
        directives_map.insert(PathBuf::from("file1.rst"), file1_map);

        let mut link_graph = LinkGraph::new();
        let mut d2_node_data = LinkNodeData::default();
        let mut d2_incoming = HashMap::new();
        d2_incoming.insert("links_to_back".to_string(), vec!["d1".to_string()]);
        d2_node_data.incoming_links = d2_incoming;
        link_graph.insert("d2".to_string(), d2_node_data);
        
        let mut d1_node_data = LinkNodeData::default();
        let mut d1_outgoing = HashMap::new();
        d1_outgoing.insert("links_to".to_string(), vec!["d2".to_string()]);
        d1_node_data.outgoing_links = d1_outgoing;
        link_graph.insert("d1".to_string(), d1_node_data);


        let aggregator = Aggregator::new(output_path, GroupBy::All);
        let output_files = aggregator.aggregate_map_to_json_with_links(&directives_map, &link_graph).unwrap();

        assert_eq!(output_files.len(), 1);
        let all_directives_file = output_path.join("all_directives.json");
        assert!(all_directives_file.exists());

        let content: Vec<DirectiveOutput> = 
            serde_json::from_str(&fs::read_to_string(all_directives_file).unwrap()).unwrap();
        assert_eq!(content.len(), 2);

        let output_d1 = content.iter().find(|d| d.id == "d1").unwrap();
        let _output_d2 = content.iter().find(|d| d.id == "d2").unwrap(); // Prefixed with _

        assert_eq!(output_d1.options.get("links_to").unwrap(), "d2");
        assert!(output_d1.options.get("links_to_back").is_none());

        // output_d1 is already defined and checked.
        // output_d2 was used to ensure its presence and check its backlinks.
        // Re-finding output_d1 is redundant.
        // The check for output_d2's options is what matters.
        let final_output_d2 = content.iter().find(|d| d.id == "d2").unwrap();

        assert!(final_output_d2.options.get("links_to").is_none()); // d2 has no outgoing "links_to"
        assert_eq!(final_output_d2.options.get("links_to_back").unwrap(), "d1");
    }
}
