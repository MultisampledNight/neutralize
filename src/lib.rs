mod messy;
mod resolve;

pub use messy::{LinkedScheme, MessyScheme};
pub use resolve::ResolvedScheme;

use {
    indexmap::IndexMap,
    itertools::Itertools,
    serde::Serialize,
    std::{collections::HashSet, fmt},
    thiserror::Error,
};

#[derive(Clone, Debug, Serialize)]
pub struct Metadata {
    pub name: String,
    pub author: String,
    pub description: Option<String>,
}

pub type Map<T, U> = IndexMap<T, U>;
pub type Set<T> = HashSet<T>;

// TODO: probably want to change this to something more specific
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Color(pub String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct SlotName(pub String);

impl fmt::Display for SlotName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct MultipleSlotNames(pub Vec<SlotName>);

impl From<Vec<SlotName>> for MultipleSlotNames {
    fn from(source: Vec<SlotName>) -> Self {
        Self(source)
    }
}

impl fmt::Display for MultipleSlotNames {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, name) in self.0.iter().enumerate() {
            if i == 0 {
                write!(f, "{}", name)?;
            } else {
                write!(f, ", {}", name)?;
            }
        }
        write!(f, "]")
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Contains(Color),
    LinkedTo(SlotName),
}

impl From<Color> for Value {
    fn from(color: Color) -> Self {
        Self::Contains(color)
    }
}

impl From<SlotName> for Value {
    fn from(name: SlotName) -> Self {
        Self::LinkedTo(name)
    }
}

pub(crate) fn collect_or_errors<A, T, E, I>(iter: I) -> Result<A, Vec<E>>
where
    I: Iterator<Item = Result<T, E>>,
    A: Default + Extend<T>,
{
    let (ok, err): (A, Vec<E>) = iter.partition_result();
    if !err.is_empty() {
        Err(err)
    } else {
        Ok(ok)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    SerdeYamlError(#[from] serde_yaml::Error),
    LinkError(Vec<messy::EmptyValueError>),
    ResolveError(Vec<resolve::Error>),
}

impl From<Vec<messy::EmptyValueError>> for Error {
    fn from(source: Vec<messy::EmptyValueError>) -> Self {
        Self::LinkError(source)
    }
}

impl From<Vec<resolve::Error>> for Error {
    fn from(source: Vec<resolve::Error>) -> Self {
        Self::ResolveError(source)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SerdeYamlError(err) => {
                writeln!(f, "Could not de/serialize scheme YAML: {}", err)?;
            }
            Self::LinkError(errors) => {
                writeln!(f, "Could not meaningfully structure scheme:")?;
                for err in errors {
                    writeln!(f, "\t{}", err)?;
                }
            }
            Self::ResolveError(errors) => {
                writeln!(f, "Could not resolve scheme:")?;
                for err in errors {
                    writeln!(f, "\t{}", err)?;
                }
            }
        }

        Ok(())
    }
}

/// Expects a scheme adhering to the [base17](https://github.com/base16-project/base17) spec,
/// outputs a YAML file with two sections: `meta` with some general information about the scheme,
/// and `slots` with all slots the scheme contained.
pub fn resolve_yaml(source: String) -> Result<String, Error> {
    let scheme: MessyScheme = serde_yaml::from_str(&source)?;
    let scheme: LinkedScheme = scheme.try_into()?;
    let scheme: ResolvedScheme = scheme.try_into()?;
    let scheme = serde_yaml::to_string(&scheme)?;
    Ok(scheme)
}
