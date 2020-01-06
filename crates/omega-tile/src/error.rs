use texture_synthesis as ts;

#[derive(Debug)]
pub enum Error {
    TsError(ts::Error),
    Io(std::io::Error),
    SizeMismatch,
    ParseError(String),
    General((Box<Error>, String))
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TsError(e) => write!(f, "{}", e),
            Self::Io(e) => write!(f, "{}", e),
            Self::SizeMismatch => write!(f, "Size mismatched"),
            Self::ParseError(s) =>  write!(f, "Parse error: {}", s),
            Self::General((err, reason)) =>  write!(f, "{} {}", reason, err),
        }
    }
}

impl From<ts::Error> for Error {
    fn from(error: ts::Error) -> Self {
        Error::TsError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}
