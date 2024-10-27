#[derive(Debug)]
pub struct ParseError(pub Box<str>);
impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for ParseError {}
impl ParseError {
    pub fn empty() -> Self {
        ParseError(Box::from(""))
    }
}
impl From<core::num::ParseIntError> for ParseError {
    fn from(value: core::num::ParseIntError) -> Self {
        Self(value.to_string().into_boxed_str())
    }
}
impl From<core::num::ParseFloatError> for ParseError {
    fn from(value: core::num::ParseFloatError) -> Self {
        Self(value.to_string().into_boxed_str())
    }
}
