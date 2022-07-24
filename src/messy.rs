use {
    super::{collect_or_errors, Color, Map, Metadata, SlotName, Value},
    serde::Deserialize,
    thiserror::Error,
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

#[derive(Debug, Error)]
#[error(
    "key '{key}' had an empty value, which can neither link to another slot nor specify a color"
)]
pub struct EmptyValueError {
    pub key: String,
}

impl TryFrom<MessyScheme> for LinkedScheme {
    type Error = Vec<EmptyValueError>;

    fn try_from(source: MessyScheme) -> Result<Self, Vec<EmptyValueError>> {
        Ok(Self {
            meta: Metadata {
                name: source.name,
                author: source.author,
                description: source.description,
            },
            slots: collect_or_errors(
                source
                    .variables
                    .into_iter()
                    .chain(source.r#override.into_iter())
                    .chain(source.palette.into_iter())
                    .map(|(key, value)| {
                        Ok((
                            SlotName(key.clone()),
                            match value.chars().next() {
                                None => return Err(EmptyValueError { key }),
                                Some('#') => Value::Contains(Color(value)),
                                _ => Value::LinkedTo(SlotName(value)),
                            },
                        ))
                    }),
            )?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct LinkedScheme {
    pub meta: Metadata,

    /// base17 doesn't describe this super-well, but basically all those sections boil down to
    /// one big hashmap, where every key is called a "slot". Then, once in that pool, we can
    /// resolve as we want. Note that base17 is still under heavy thoughtwork though.
    pub slots: Map<SlotName, Value>,
}
