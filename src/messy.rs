use {
    super::{Map, Metadata, SlotName, Value},
    serde::Deserialize,
};

#[derive(Clone, Debug, Deserialize)]
pub struct MessyScheme {
    pub name: String,
    pub author: String,
    pub description: Option<String>,

    pub variables: Map<String, String>,
    pub r#override: Map<String, String>,
    pub palette: Map<String, String>,
}

pub struct LinkedScheme {
    pub meta: Metadata,

    /// base17 doesn't describe this super-well, but basically all those sections boil down to
    /// one big hashmap, where every key is called a "slot". Then, once in that pool, we can
    /// resolve as we want. Note that base17 is still under heavy thoughtwork though.
    pub slots: Map<SlotName, Value>,
}
