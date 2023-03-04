use std::num::ParseFloatError;

use std::num::ParseIntError;

use miette::Diagnostic;
use thiserror::Error;
use winnow;

use miette::SourceSpan;
use winnow::error::FromExternalError;
use winnow::Parser;

// use crate::formats::util::ParseErrorKind;
// use crate::formats::util::SyntaxParseError;

use crate::formats::util::word;

use super::mesh;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("Syntax error")]
    #[diagnostic(code(fds_tbx::smv::generic_syntax), help("error: {kind:#?}"))]
    Syntax {
        #[label("here")]
        location: SourceSpan,
        kind: winnow::error::ErrorKind,
    },
    #[error("Syntax error")]
    #[diagnostic(code(fds_tbx::smv::generic_syntax_nd))]
    SyntaxNonDiagnostic {
        remaining_length_bytes: usize,
        kind: winnow::error::ErrorKind,
    },
    #[error("File ended early, expected at least {0:#?} more bytes")]
    #[diagnostic(
        code(fds_tbx::smv::early_eof),
        help("expected at least {0:#?} more bytes")
    )]
    Incomplete(winnow::error::Needed),
    #[error("Missing sub-section {name}")]
    #[diagnostic(code(fds_tbx::smv::missing_sub_section))]
    MissingSubSection {
        #[label("in this section")]
        parent: SourceSpan,
        #[help("expected a sub-section {name}")]
        name: &'static str,
        #[label("found this instead")]
        found: Option<SourceSpan>,
    },
    #[error("Missing section {name}")]
    #[diagnostic(code(fds_tbx::smv::missing_section))]
    MissingSection {
        #[help("expected a section {name}")]
        name: &'static str,
    },
    #[error("Reference to undefined {key_type}")]
    #[diagnostic(code(fds_tbx::smv::invalid_key))]
    InvalidKey {
        #[label("this key is invalid")]
        key: SourceSpan,
        #[help("expected a reference to a {name}")]
        key_type: &'static str,
    },
    #[error("Found an index out of order")]
    #[diagnostic(code(fds_tbx::smv::suspicious_index))]
    SuspiciousIndex {
        #[label("inside this sub-section")]
        inside_subsection: SourceSpan,
        #[label("this index is suspicious")]
        index: SourceSpan,
        #[help("expected an index of {expected}")]
        expected: usize,
    },
    #[error("Obst ID is of sign {signum}. Expected -1 or 1")]
    #[diagnostic(code(fds_tbx::smv::unexpected_obst_id_sign))]
    UnexpectedObstIdSign {
        #[label("this mesh id")]
        number: SourceSpan,
        #[help("is of sign {signum}. Expected -1 or 1")]
        signum: i32,
    },
    #[error("Either a color index or an RGB color must be specified")]
    #[diagnostic(code(fds_tbx::smv::invalid_obst_color))]
    InvalidObstColor {
        #[label("specified index")]
        color_index: SourceSpan,
        #[label("specified RGB")]
        rgb: Option<SourceSpan>,
    },
    #[error("Texture origin must be specified on all non-dummy vents, and must not be specified on dummy vents")]
    #[diagnostic(code(fds_tbx::smv::vent_missing_texture_origin))]
    // #[help("Expected a texture origin for vent {vent_index} of {num_vents_total} vents, because it is within the first {num_non_dummies} vents, which are not dummy vents and must therefore have a texture origin specified.")]
    VentTextureOrigin {
        #[label("this vent")]
        vent: SourceSpan,
        #[help("theres {num_vents_total} vents in total")]
        num_vents_total: usize,
        #[help("the first {num_non_dummies} vents are non-dummy vents and must have a texture origin specified")]
        num_non_dummies: usize,
        #[help("expected a texture origin for vent {vent_line_number}")]
        vent_line_number: usize,
        #[label("this is the texture origin")]
        texture_origin: Option<SourceSpan>,
    },
    #[error("Encountered unknown section")]
    #[diagnostic(code(fds_tbx::smv::unknown_section))]
    UnknownSection {
        #[label("here")]
        section: SourceSpan,
    },
}

impl From<winnow::error::ErrMode<winnow::error::Error<&str>>> for Error {
    fn from(err: winnow::error::ErrMode<winnow::error::Error<&str>>) -> Self {
        // Self::SyntaxNonDiagnostic(err.map_input(|e| e.into()))
        match err {
            winnow::error::ErrMode::Incomplete(needed) => Self::Incomplete(needed),
            winnow::error::ErrMode::Backtrack(err) | winnow::error::ErrMode::Cut(err) => {
                Self::SyntaxNonDiagnostic {
                    remaining_length_bytes: err.input.len(),
                    kind: err.kind,
                }
            }
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
// #[error("oops!")]
// #[diagnostic(
//     // code(oops::my::bad),
//     // url(docsrs),
//     // help("try doing it better next time?")
// )]
pub enum ErrorOld {
    #[error("oops!")]
    WrongSyntax {
        #[label("here")]
        pos: SourceSpan,
        #[source]
        #[diagnostic_source]
        err: ErrorKind,
    },
    // #[error("oops!")]
    // Nom(#[source] winnow::error::ErrMode<winnow::error::Error<String>>),
    // TODO: Using enum instead of a &str worth it?
    #[error("oops!")]
    MissingSection { name: &'static str },
    #[error("oops!")]
    MissingSubSection {
        parent: SourceSpan,
        name: &'static str,
    },
    #[error("oops!")]
    InvalidKey { parent: SourceSpan, key: SourceSpan },
    // #[error(transparent)]
    // #[diagnostic(code(oops::my::bad))]
    // SyntaxError(#[from] SyntaxParseError<String>),
}

#[derive(Debug, Error, Diagnostic)]
pub enum ErrorKind {
    /// An error occurred while parsing an integer.
    #[error(transparent)]
    #[diagnostic(code(fds_tbx::smv::parse_int))]
    ParseIntError(ParseIntError),

    /// An error occurred while parsing a floating point number.
    #[error(transparent)]
    #[diagnostic(code(fds_tbx::smv::parse_float))]
    ParseFloatError(ParseFloatError),
    // // TODO: Expected is currently a lower bound, not an exact value because of the way the macro is written
    // WrongNumberOfValues { expected: usize, got: usize },
    // TrailingCharacters,
    // UnknownSection,
    // MismatchedIndex { expected: usize, got: usize },
    // Mesh(mesh::ErrorKind),
    // InvalidSection,
}

// impl<'a> FromExternalError<&'a str, ParseIntError> for SyntaxParseError<&'a str, ErrorKind> {
//     fn from_external_error(input: &'a str, _kind: winnow::error::ErrorKind, e: ParseIntError) -> Self {
//         SyntaxParseError {
//             input,
//             len: 0,
//             label: None,
//             help: None,
//             context: None,
//             kind: Some(ErrorKind::ParseIntError(e)),
//             touched: false,
//         }
//     }
// }

// impl<'a> FromExternalError<&'a str, ParseFloatError> for SyntaxParseError<&'a str, ErrorKind> {
//     fn from_external_error(input: &'a str, _kind: winnow::error::ErrorKind, e: ParseFloatError) -> Self {
//         SyntaxParseError {
//             input,
//             len: 0,
//             label: None,
//             help: None,
//             context: None,
//             kind: Some(ErrorKind::ParseFloatError(e)),
//             touched: false,
//         }
//     }
// }
