use miette::{Diagnostic, SourceCode};
use thiserror::Error;
use winnow;

use miette::SourceSpan;

/// A parsing error encountered while parsing a ".smv" file.
/// For documentation on the individual errors, look at the `#[diagnostic(help(...))]` attributes.
#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("Syntax error")]
    #[diagnostic(code(fds_tbx::smv::generic_syntax), help("error kind: {kind:#?}"))]
    Syntax {
        #[label("here")]
        location: SourceSpan,
        kind: winnow::error::ErrorKind,
    },
    // TODO: This is a workaround, find a fix.
    //       Maybe a secondary enum that has `SyntaxNonDiagnostic { [this] }`
    //        and `Other(Error)`, implementing `From<...>` for both?
    //
    // This value should not be returned as final value, and gets replaced by `SimulationParser::map_err`
    //
    // It's a bit of a workaround for converting `winnow` errors to nice pretty-printable errors:
    //  When `winnow` returns a syntax error it gives the length of the remaining input,
    //  but `miette`, which we use for pretty-printing the error, takes `SourceSpan` (or `impl Into<SourceSpan>`).
    //  To get a `SourceSpan` we need the offset from the start of the source, but we only have the offset from the end.
    //  In order to compute the offset from the start, we need the length of the entire source.
    //  Since we want to implement `From<[winnow error type]>`, we can't get information about the entire source.
    //  To solve this, this value exists to store the information we have at the time of calling `From::from`
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
        color: Option<SourceSpan>,
    },
    #[error("Texture origin must be specified on all non-dummy vents, and must not be specified on dummy vents")]
    #[diagnostic(code(fds_tbx::smv::vent_missing_texture_origin))]
    VentTextureOrigin {
        #[label("in this vent")]
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
    #[diagnostic(code(fds_tbx::smv::unknown_section), severity(warn))]
    UnknownSection {
        #[label("here")]
        section: SourceSpan,
    },
    #[error("Found wrong number of meshes, got {found}, expected {expected}")]
    #[diagnostic(
        code(fds_tbx::smv::wrong_number_of_meshes),
        help("expected {expected} meshes, found {found} meshes")
    )]
    WrongNumberOfMeshes { expected: usize, found: usize },
    #[error("Found unexpected constant value, expected {expected}")]
    #[diagnostic(code(fds_tbx::smv::unexpected_constant_value))]
    InvalidFloatConstant {
        #[label("expected {expected} here")]
        span: SourceSpan,
        expected: f32,
    },
    #[error("Found unexpected constant value, expected {expected}")]
    #[diagnostic(code(fds_tbx::smv::unexpected_constant_value))]
    InvalidIntConstant {
        #[label("expected {expected} here")]
        span: SourceSpan,
        expected: i32,
    },
    #[error("Found end of input unexpectedly")]
    #[diagnostic(code(fds_tbx::smv::unexpected_end_of_input))]
    UnexpectedEndOfInput {
        #[label("here")]
        span: SourceSpan,
    },
}

impl From<winnow::error::ErrMode<winnow::error::Error<&str>>> for Error {
    fn from(err: winnow::error::ErrMode<winnow::error::Error<&str>>) -> Self {
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

impl Error {
    /// Converts the given [`err::Error`] into a pretty-printable [`miette::Report`].
    pub fn add_src<Src: SourceCode + Send + Sync + 'static>(
        self,
        owned_input: Src,
    ) -> miette::Report {
        miette::Report::new(self).with_source_code(owned_input)
    }
}