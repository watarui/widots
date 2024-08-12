use std::borrow::Cow;

#[derive(Clone, Debug)]
pub enum Cmd<'a> {
    Empty,
    Cmd(Cow<'a, str>),
}

#[derive(Clone, Debug)]
pub enum Argument<'a> {
    Empty,
    Arg(Cow<'a, str>),
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
