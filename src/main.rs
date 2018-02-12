extern crate config;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate wordsapi_client;

use std::path::PathBuf;
use structopt::StructOpt;
use wordsapi_client::{WordData, WordResponse};
use std::env;
use std::fs;
use std::io;
use std::io::Write;

#[derive(StructOpt, Debug)]
#[structopt(name = "word", about = "Look up a word.")]
struct Opt {
    #[structopt(short = "d", long = "debug", help = "Activate debug mode")]
    debug: bool,
    #[structopt(short = "j", long = "json", help = "Output raw json")]
    json: bool,
    #[structopt(help = "The word to look up")]
    word: String,
    #[structopt(help = "API token, from environment if not present")]
    token: Option<String>,
}

fn main() {
    let opt = Opt::from_args();

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Settings"))
        .unwrap()
        .merge(config::Environment::with_prefix("WORD"))
        .unwrap();
    let token = settings.get_str("token").unwrap();
    let word_client = wordsapi_client::WordClient::new(&token);
    let result = word_client.look_up(&opt.word);
    match result {
        Ok(wr) => handle(wr, &opt),
        Err(e) => println!("Got an error {}", e),
    }
}

fn handle(mut response: WordResponse, opt: &Opt) {
    if opt.json {
        display_json(&mut response)
    } else {
        /*
        let data = response.try_parse();
        match data {
            Ok(mut wd) => display_definition(&mut wd),
            Err(e) => println!("WordAPI Clent response error: {}", e),
        }
        */
        let json = response.raw_json();
        write_to_cache(&json, &opt);
    }
}

fn display_json(response: &mut WordResponse) {
    println!("{}", response.raw_json());
}

fn write_to_cache(json: &str, opt: &Opt) -> Result<(), io::Error> {
    let cache_dir = match env::home_dir() {
        Some(path) => path.join(".word"),
        None => PathBuf::from("."),
    };
    println!("cache_dir is {}", cache_dir.display());
    fs::create_dir_all(&cache_dir);
    let mut dest = {
        let fname = format!("{}.json", &opt.word);
        println!("saving using file name: '{}'", fname);
        let fname = cache_dir.join(fname);
        println!("will be located under: '{:?}'", fname);
        // create file with given name inside the temp dir
        fs::File::create(fname)?
    };
    // data is copied into the target file
    dest.write_all(json.as_bytes());
    Ok(())
}

fn display_definition(word: &WordData) {
    println!("{} |{}|", &word.word, pronunciation(word));
    for e in &word.results {
        println!("   {}: {}", e.part_of_speech, e.definition);
    }
}

fn pronunciation(word: &WordData) -> &str {
    let p = word.pronunciation.get("all");
    match p {
        Some(p) => p,
        None => "",
    }
}
