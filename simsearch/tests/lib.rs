use std::fs::File;
use std::io::Read;

use itertools::Itertools;
use json;
use simsearch::SimSearch;

#[macro_use(quickcheck)]
extern crate quickcheck_macros;
#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref ENGINE: SimSearch<String> = populate_engine();
}

fn populate_engine() -> SimSearch<String> {
    let mut engine = SimSearch::new();

    let mut file = File::open("./books.json").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let j = json::parse(&content).unwrap();

    for book in j.members() {
        let title = book["title"].as_str().unwrap();
        engine.insert(
            title.to_owned(),
            &title.split_whitespace().collect::<Vec<&str>>(),
        );
    }

    engine
}

#[quickcheck]
fn test_quickcheck(tokens: Vec<String>) {
    ENGINE.search(&tokens.join(" "));
}
