use std::fs::File;
use std::io::{self, Read as _};
use std::time::Instant;

use json;
use simsearch::SimSearch;

fn main() -> io::Result<()> {
    let mut engine = SimSearch::new();

    let mut file = File::open("./books.json")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let j = json::parse(&content).unwrap();

    for book in j.members() {
        let title = book["title"].as_str().unwrap();
        engine.insert(
            title.to_owned(),
            &title.split_whitespace().collect::<Vec<&str>>(),
        );
    }

    println!("Please input a query string and hit enter (e.g 'old man'):",);

    loop {
        let mut pattern = String::new();
        io::stdin()
            .read_line(&mut pattern)
            .expect("failed to read from stdin");

        let start = Instant::now();
        let pattern = pattern.replace("\r\n", "");
        let end = Instant::now();

        println!("pattern: {:?}", &pattern);
        println!("results: {:?}", engine.search(&pattern));
        println!("time: {:?}", end - start);
    }
}
