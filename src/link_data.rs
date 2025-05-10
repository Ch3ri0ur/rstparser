use serde::Deserialize;
use std::collections::HashMap;

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
