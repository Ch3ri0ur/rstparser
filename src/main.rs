mod parser;
mod file_walker;
mod aggregator;
mod processor;
mod extractor;

use std::path::PathBuf;
use std::process;
use clap::{Parser, ValueEnum};
use file_walker::FileWalker;
use processor::Processor;
use aggregator::{Aggregator, GroupBy};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Directory to search for RST files
    #[arg(short, long, default_value = ".")]
    dir: String,

    /// File extensions to search (comma-separated)
    #[arg(short, long, default_value = "rst")]
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
    
    // Parse extensions
    let extensions: Vec<String> = cli.extensions
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    
    // Parse directive names
    let directives: Vec<String> = cli.directives
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    
    if directives.is_empty() {
        eprintln!("Error: At least one directive name must be specified");
        process::exit(1);
    }
    
    // Create output directory path
    let output_dir = PathBuf::from(&cli.output);
    
    // Find RST files
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
    
    // Process files to find directives
    let processor = Processor::new(directives);
    let directives_with_source = match processor.process_files(files) {
        Ok(directives) => directives,
        Err(err) => {
            eprintln!("Error processing files: {}", err);
            process::exit(1);
        }
    };
    
    println!("Found {} directives", directives_with_source.len());
    
    // Aggregate directives to JSON files
    let aggregator = Aggregator::new(output_dir, cli.group_by.into());
    match aggregator.aggregate_to_json(directives_with_source) {
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
