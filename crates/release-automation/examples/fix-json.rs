//! from https://github.com/rust-lang/rustfix/blob/master/examples/fix-json.rs#L8

use anyhow::Error;
use std::io::{stdin, BufReader, Read};
use std::{collections::HashMap, collections::HashSet, env, fs};

fn main() -> Result<(), Error> {
    let suggestions_file = env::args().nth(1).expect("USAGE: fix-json <file or -->");
    let suggestions = if suggestions_file == "--" {
        let mut buffer = String::new();
        BufReader::new(stdin()).read_to_string(&mut buffer)?;
        buffer
    } else {
        fs::read_to_string(&suggestions_file)?
    };

    let suggestions_json: Vec<serde_json::Value> = serde_json::from_str(&suggestions).unwrap();
    let suggestions_json_filtered: Vec<serde_json::Value> = suggestions_json
        .into_iter()
        .filter(|v| {
            if let serde_json::Value::Object(m) = v {
                m.contains_key("message")
            } else {
                false
            }
        })
        .collect();
    let suggestions_filtered = serde_json::to_string_pretty(&suggestions_json_filtered)?;

    std::fs::write("filtered.json", &suggestions_filtered)?;

    let suggestions = rustfix::get_suggestions_from_json(
        &suggestions_filtered,
        // &suggestions,
        &HashSet::new(),
        rustfix::Filter::MachineApplicableOnly,
    )
    .unwrap();

    let mut files = HashMap::new();
    for suggestion in suggestions {
        let file = suggestion.solutions[0].replacements[0]
            .snippet
            .file_name
            .clone();
        files.entry(file).or_insert_with(Vec::new).push(suggestion);
    }

    for (source_file, suggestions) in &files {
        let source = fs::read_to_string(source_file)?;
        let mut fix = rustfix::CodeFix::new(&source);
        for suggestion in suggestions.iter().rev() {
            if let Err(e) = fix.apply(suggestion) {
                eprintln!("Failed to apply suggestion to {}: {}", source_file, e);
            }
        }
        let fixes = fix.finish()?;
        fs::write(source_file, fixes)?;
    }

    Ok(())
}
