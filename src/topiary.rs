//! Utility module for formatting OpenSCAD files using Topiary

use std::{
    fmt::Display,
    io::{Read, Write},
};

use topiary_core::{formatter, Language, Operation, TopiaryQuery};

#[derive(Debug)]
/// Errors that may be encountered during formatting
pub struct FormatError(topiary_core::FormatterError);

impl Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

const OPENSCAD_QUERY: &str = include_str!("../openscad.scm");

/// Format an Openscad file being read from `input`, writing the result to `output`.
pub fn format(
    mut input: impl Read,
    mut output: impl Write,
    indent: Option<String>,
    query_str: Option<&str>,
) -> Result<(), FormatError> {
    let query_str = query_str.unwrap_or(OPENSCAD_QUERY);
    let grammar = tree_sitter_openscad::LANGUAGE.into();
    let query = TopiaryQuery::new(&grammar, query_str).map_err(FormatError)?;
    let language = Language {
        name: "openscad".to_owned(),
        query,
        grammar,
        indent,
    };

    formatter(
        &mut input,
        &mut output,
        &language,
        Operation::Format {
            // We only enable the idempotency check in debug mode: it's useful to detect bugs in
            // the Nickel formatter, but we don't want to report an error or to make production
            // users pay the cost of the check, although this cost should be fairly low.
            skip_idempotence: !cfg!(debug_assertions),
            tolerate_parsing_errors: false,
        },
    )
    .map_err(FormatError)
}
