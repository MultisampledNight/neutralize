use {
    serde::{Deserialize, Serialize},
    std::{collections::BTreeMap, env, fs},
};

#[derive(Clone, Debug, Serialize)]
struct Metadata {
    name: String,
    author: String,
    description: Option<String>,
}

type Map<T, U> = BTreeMap<T, U>;

#[derive(Clone, Debug, Deserialize)]
struct MessyScheme {
    name: String,
    author: String,
    description: Option<String>,

    variables: Map<String, String>,
    r#override: Map<String, String>,
    palette: Map<String, String>,
}

// TODO: probably want to change this to something more specific
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
struct Color(pub String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
struct SlotName(pub String);

#[derive(Clone, Debug)]
enum Value {
    Contains(Color),
    LinkedTo(SlotName),
}

/// base17 doesn't describe this super-well, but
struct StructuredScheme {
    meta: Metadata,
    slots: Map<SlotName, Value>,
}

#[derive(Serialize)]
struct ResolvedScheme {
    meta: Metadata,
    slots: Map<SlotName, Color>,
}

fn main() {
    let scheme: MessyScheme =
        serde_yaml::from_str(&fs::read_to_string(env::args().skip(1).next().unwrap()).unwrap())
            .unwrap();
    dbg!(scheme);
}
