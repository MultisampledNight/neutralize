use {
    neutralize::{LinkedScheme, MessyScheme},
    std::{env, fs},
};

fn main() {
    let scheme: MessyScheme =
        serde_yaml::from_str(&fs::read_to_string(env::args().skip(1).next().unwrap()).unwrap())
            .unwrap();
    let scheme: LinkedScheme = scheme.try_into().unwrap();
    dbg!(scheme);
}
