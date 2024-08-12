use std::borrow::Cow;

/// Represents a shell command.
#[derive(Clone, Debug)]
pub enum Cmd<'a> {
    /// An empty command.
    Empty,
    /// A non-empty command.
    Cmd(Cow<'a, str>),
}

/// Represents command arguments.
#[derive(Clone, Debug)]
pub enum Argument<'a> {
    /// No arguments.
    Empty,
    /// A single argument.
    Arg(Cow<'a, str>),
    /// Multiple arguments.
    Args(Vec<Cow<'a, str>>),
}

impl<'a> From<&'a str> for Cmd<'a> {
    fn from(s: &'a str) -> Self {
        if s.is_empty() {
            Cmd::Empty
        } else {
            Cmd::Cmd(Cow::Borrowed(s))
        }
    }
}

impl<'a> From<&'a str> for Argument<'a> {
    fn from(s: &'a str) -> Self {
        if s.is_empty() {
            Argument::Empty
        } else {
            Argument::Arg(Cow::Borrowed(s))
        }
    }
}

impl<'a> From<Vec<&'a str>> for Argument<'a> {
    fn from(v: Vec<&'a str>) -> Self {
        if v.is_empty() {
            Argument::Empty
        } else {
            Argument::Args(v.into_iter().map(Cow::Borrowed).collect())
        }
    }
}

impl<'a> Cmd<'a> {
    /// Creates a new `Cmd` instance.
    ///
    /// # Arguments
    ///
    /// * `s` - The command string
    ///
    /// # Returns
    ///
    /// A new `Cmd` instance.
    pub fn new<S: Into<Cow<'a, str>>>(s: S) -> Self {
        Cmd::Cmd(s.into())
    }
}
