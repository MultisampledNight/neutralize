mod messy;
mod resolve;

pub use messy::{LinkedScheme, MessyScheme};
pub use resolve::ResolvedScheme;

use {itertools::Itertools, serde::Serialize, std::collections::BTreeMap};

#[derive(Clone, Debug, Serialize)]
pub struct Metadata {
    pub name: String,
    pub author: String,
    pub description: Option<String>,
}

pub type Map<T, U> = BTreeMap<T, U>;

// TODO: probably want to change this to something more specific
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Color(pub String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct SlotName(pub String);

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
