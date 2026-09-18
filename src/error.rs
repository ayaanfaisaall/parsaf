use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum ParsafError {
    #[error("token not allowed here")]
    #[diagnostic(
        code(afsh::parsaf::not_allowed),
        help("remove '{token}' or provide a valid expression before it")
    )]
    NotAllowedHere {
        token: String,
        #[label("'{token}' cannot be placed at the start of a statement")]
        span: SourceSpan,
    },

    #[error("unexpected token found")]
    #[diagnostic(
        code(afsh::parsaf::unexpected_token),
        help("use one of 'datatypes', or try using: \"\" or {{}}")
    )]
    BaseCase {
        token: String,
        #[label("'{token}' is unexpected here")]
        span: SourceSpan,
    },

    #[error("unexpected token found")]
    #[diagnostic(
        code(afsh::parsaf::unexpected_token),
        help("remove '{token}' and add the correct token")
    )]
    UnexpectedToken {
        token: String,
        #[label("'{token}' is unexpected here")]
        span: SourceSpan,
    },

    #[error("unclosed delimiter: '{delimiter}'")]
    #[diagnostic(
        code(afsh::parsaf::unclosed_delimiter),
        help("close: '{delimiter}' properly")
    )]
    UnclosedDelimiter {
        delimiter: String,
        #[label("'{delimiter}' might not be closed properly")]
        span: SourceSpan,
    },

    #[error("missing: '{expected}'")]
    #[diagnostic(
        code(afsh::parsaf::expected_found),
        help("try adding: '{expected}' here")
    )]
    ExpectedFound {
        expected: String,
        found: String,
        #[label("expected '{expected}' here")]
        span: SourceSpan,
    },

    #[error("unexpected EOF")]
    #[diagnostic(
        code(afsh::parsaf::unexpected_eof),
        help("you might be missing a closing brace '}}' or bracket ']'?")
    )]
    UnexpectedEof,
}
