mod messy;
mod resolve;

pub use messy::{LinkedScheme, MessyScheme};
pub use resolve::ResolvedScheme;

use {serde::Serialize, std::collections::BTreeMap};

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
