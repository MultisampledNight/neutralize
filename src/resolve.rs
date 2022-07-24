use {
    super::{Color, Map, Metadata, SlotName},
    serde::Serialize,
};

#[derive(Serialize)]
pub struct ResolvedScheme {
    pub meta: Metadata,
    pub slots: Map<SlotName, Color>,
}
