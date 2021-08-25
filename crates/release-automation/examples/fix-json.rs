//! from https://github.com/rust-lang/rustfix/blob/master/examples/fix-json.rs#L8

#![allow(unused_imports)]

use anyhow::Error;
use std::io::{stdin, BufReader, Read};
use std::path::PathBuf;
use std::{collections::HashMap, collections::HashSet, env, fs};

mod types {
    use serde::Deserialize;
    use serde_with::{serde_as, DefaultOnError};
    use std::path::PathBuf;

    #[derive(Debug, Deserialize)]
    #[serde(tag = "reason")]
    #[serde(rename = "kebab-case")]
    pub struct CompilerMessage {
        // pub package_id: String,
        // pub manifest_path: String,
        // pub target: Target,
        pub message: rustfix::diagnostics::Diagnostic,
    }

    // #[derive(Debug, Deserialize)]
    // pub struct Message {
    //     pub children: Vec<Message>,
    //     pub code: Option<String>,
    //     pub message: String,
    //     pub rendered: Option<String>,
    //     pub spans: Vec<Span>,
    // }

    // #[derive(Debug, Deserialize)]
    // pub struct Span {
    //     pub byte_end: usize,
    //     pub byte_start: usize,
    //     pub column_end: usize,
    //     pub column_start: usize,
    //     // TODO: double-check the type because i only found "null"s
    //     pub expansion: Option<String>,
    //     pub file_name: PathBuf,
    //     pub is_primary: bool,
    //     // TODO: double-check the type because i only found "null"s
    //     pub label: Option<String>,
    //     pub line_end: usize,
    //     pub line_start: usize,
    //     pub suggested_replacement: Option<String>,
    //     // TODO: double-check the type because i only found "null"s
    //     pub suggestion_applicability: Option<String>,
    //     pub text: Vec<SpanText>,
    // }

    // #[derive(Debug, Deserialize)]
    // pub struct SpanText {
    //     pub highlight_end: usize,
    //     pub highlight_start: usize,
    //     pub text: String,
    // }

    // #[derive(Debug, Deserialize)]
    // pub struct Target {
    //     pub kind: Vec<String>,
    //     pub crate_types: Vec<String>,
    //     pub name: String,
    //     pub src_path: String,
    //     pub edition: String,
    //     pub doc: bool,
    //     pub doctest: bool,
    //     pub test: bool,
    // }

    // #[derive(Debug, Deserialize)]
    // pub struct Profile {
    //     pub opt_level: String,
    //     pub debuginfo: Option<usize>,
    //     pub debug_assertions: bool,
    //     pub overflow_checks: bool,
    //     pub test: bool,
    // }
}

fn main() -> Result<(), Error> {
    let json_file = env::args().nth(1).expect("USAGE: fix-json <file or -->");
    let json = if json_file == "--" {
        let mut buffer = String::new();
        BufReader::new(stdin()).read_to_string(&mut buffer)?;
        buffer
    } else {
        fs::read_to_string(&json_file)?
    };

    // we're only interested in the compiler-message variant, so we can use a single struct.
    // see: https://stackoverflow.com/questions/67702612/how-to-ignore-unknown-enum-variant-while-deserializing
    use serde::Deserialize;
    use serde_with::{serde_as, DefaultOnError};
    #[serde_as]
    #[derive(serde::Deserialize)]
    pub struct W(#[serde_as(as = "Vec<DefaultOnError>")] pub Vec<Option<types::CompilerMessage>>);

    // Convert the JSON string to vec.
    let clippy_lint_elements: Vec<types::CompilerMessage> = serde_json::from_str::<W>(&json)
        .unwrap()
        .0
        .into_iter()
        .flatten()
        .collect();

    let mut path_suggestions = HashMap::<PathBuf, Vec<rustfix::Suggestion>>::new();

    let mut num_suggestions = 0;

    for m in &clippy_lint_elements {
        let mut messages = vec![&m.message];
        while let Some(message) = messages.pop() {
            messages.extend(message.children.iter());
            for span in &message.spans {
                if let Some(new_suggestion) = rustfix::collect_suggestions(
                    message,
                    &HashSet::new(),
                    rustfix::Filter::Everything,
                ) {
                    path_suggestions
                        .entry(PathBuf::from(span.file_name.clone()))
                        .or_insert_with(|| vec![new_suggestion]);
                    num_suggestions += 1;
                }
            }
        }
    }

    println!(
        "got {} lints and {} files with {} suggestions",
        clippy_lint_elements.len(),
        path_suggestions.len(),
        num_suggestions,
    );

    for (source_file, suggestions) in path_suggestions {
        let source = fs::read_to_string(&source_file)?;
        let mut fix = rustfix::CodeFix::new(&source);
        for suggestion in suggestions.iter().rev() {
            if let Err(e) = fix.apply(suggestion) {
                eprintln!("Failed to apply suggestion to {:?}: {}", &source_file, e);
            }
        }
        // let fixes = fix.finish()?;
        // fs::write(source_file, fixes)?;
    }

    Ok(())
}
