use crate::sexpr::TokInfo;

#[derive(Debug)]
pub enum ParsingError {
    /// Invalid Syntax error. The first argument is the position, the second is the token that
    /// triggered the error. And the optional string is an optional error message.
    InvalidSyntax(TokInfo, String, Option<String>),
    /// Unexpected token error. The first argument is the error produced by pest
    Pest(String),
    /// Nothing to parse error. This can be handled as not an erorr, but as a warning.
    NothingToParse,
    /// Bad wellformed config error.
    BadWellformedConfig,
}

impl std::error::Error for ParsingError {}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::InvalidSyntax(pos, token, msg) => {
                write!(f, "Invalid syntax `{}` : {}", token, pos)?;
                if let Some(msg) = msg {
                    write!(f, "Error: {}", msg)?;
                }
                Ok(())
            }
            ParsingError::Pest(msg) => write!(f, "Pest error:\n {}", msg),
            ParsingError::NothingToParse => write!(f, "Nothing to parse"),
            ParsingError::BadWellformedConfig => write!(f, "Bad wellformed config"),
        }
    }
}
