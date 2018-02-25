extern crate config;
extern crate serde_json;
#[macro_use]
extern crate structopt;
extern crate wordsapi_client;

use std::path::PathBuf;
use structopt::StructOpt;
use wordsapi_client::{WordAPIError, WordData};
use std::env;
use std::fs;
use std::io;
use std::io::{Read, Write};
use config::Config;
use std::io::Error;
use std::io::ErrorKind;

#[derive(StructOpt, Debug)]
#[structopt(name = "word", about = "Look up a word.")]
struct Opt {
    #[structopt(short = "v", long = "verbose", help = "Show verbose output")]
    verbose: bool,
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
    match load_word_json(&settings, &opt) {
        Ok(ref word_json) => match handle_word_json(&settings, &opt, word_json) {
            Ok(()) => (),
            Err(e) => println!("Could not parse word json {}", e),
        },
        Err(e) => println!("Could not load word json {}", e),
    }
}

fn handle_word_json(_settings: &Config, opt: &Opt, word_json: &str) -> Result<(), WordAPIError> {
    if opt.json {
        display_json(word_json);
        Ok(())
    } else {
        match wordsapi_client::try_parse(word_json) {
            Ok(ref word_data) => {
                display_word_data(word_data, opt);
                Ok(())
            }
            Err(_e) => Err(WordAPIError::ResultParseError),
        }
    }
}

fn load_word_json(settings: &Config, opt: &Opt) -> Result<String, Error> {
    let cache_dir = get_cache_dir();
    if opt.verbose {
        println!("cache_dir is {}", cache_dir.display());
    }
    create_cache_dir(&cache_dir);
    let cache_file_path = get_cache_file_path(&cache_dir, opt);
    match read_cache_file(&cache_file_path) {
        Ok(cached_json) => Ok(cached_json),
        Err(_e) => {
            if opt.verbose {
                println!("could not find cached json, calling service...");
            }
            match fetch_word_json(settings, opt) {
                Ok(fetched_json) => {
                    write_to_cache(&fetched_json, &cache_file_path);
                    Ok(fetched_json)
                }
                Err(e) => Err(e),
            }
        }
    }
}

fn fetch_word_json(settings: &Config, opt: &Opt) -> Result<String, Error> {
    let token = settings.get_str("token").unwrap();
    let word_client = wordsapi_client::WordClient::new(&token);
    let result = word_client.look_up(&opt.word);
    match result {
        Ok(wr) => { 
            if opt.verbose {
                println!("{} API requests remaining of {}.", &wr.rate_limit_remaining, &wr.rate_limit_requests_limit);
            }
            Ok(wr.response_json)
        },
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }
}

fn display_word_data(word_data: &WordData, _opt: &Opt) {
    display_definition(word_data);
}

fn display_json(word_json: &str) {
    println!("{}", word_json);
}

fn write_to_cache(json: &str, cache_file_path: &PathBuf) {
    match fs::File::create(cache_file_path) {
        Ok(cache_file) => write_to_cache_file(json, cache_file),
        Err(e) => println!("Warning: could not write cache file: {}", e),
    }
}

fn write_to_cache_file(json: &str, mut cache_file: std::fs::File) {
    match cache_file.write_all(json.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            println!("Warning: could not write cache file: {}", e);
        }
    }
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

fn create_cache_dir(cache_dir: &PathBuf) {
    match fs::create_dir_all(&cache_dir) {
        Ok(_) => (),
        Err(e) => {
            println!("Warning: could not create cache directory: {}", e);
            ()
        }
    }
}

fn get_cache_dir() -> PathBuf {
    match env::home_dir() {
        Some(path) => path.join(".word"),
        None => PathBuf::from("./.word"),
    }
}

fn get_cache_file_path(cache_dir: &PathBuf, opt: &Opt) -> PathBuf {
    let fname = format!("{}.json", &opt.word);
    if opt.verbose {
        println!("saving using file name: '{}'", fname);
    }
    let fname = cache_dir.join(fname);
    if opt.verbose {
        println!("will be located under: '{:?}'", fname);
    }
    fname
}

fn read_cache_file(cache_file_path: &PathBuf) -> io::Result<String> {
    let mut cache_file = fs::File::open(cache_file_path)?;
    let mut contents = String::new();
    let _size = cache_file.read_to_string(&mut contents)?;
    Ok(contents)
}

