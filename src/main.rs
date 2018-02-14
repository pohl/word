extern crate config;
#[macro_use]
extern crate structopt;
extern crate wordsapi_client;
extern crate serde_json;

use std::path::PathBuf;
use structopt::StructOpt;
use wordsapi_client::{WordData, WordResponse};
use std::env;
use std::fs;
use std::io;
use std::io:: {Read, Write};
use config::Config;
//use std::error::Error;
use std::io::Error;
use std::io::ErrorKind;

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
    let result = get_word_data(&settings, &opt.word);
    match result {
        Ok(word_data) => display_word_data(word_data, &opt),
        Err(e) => println!("Got an error {}", e),
    }
}

fn get_word_data(settings: &Config, word: &str) -> Result<WordData, Error> {
    match load_word_json(settings, word) {
        Ok(ref word_json) => wordsapi_client::try_parse(word_json),
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }
}

fn load_word_json(settings: &Config, word: &str) -> Result<String, Error> {
    let cache_dir = get_cache_dir();
    println!("cache_dir is {}", cache_dir.display());
    create_cache_dir(&cache_dir);
    let cache_file_path = get_cache_file_path(&cache_dir, &word);
    match read_cache_file(&cache_file_path) {
        Ok(cached_json) => Ok(cached_json),
        Err(e) => { 
            println!("could not find cached json, calling service...");
            match fetch_word_json(settings, word) {
                Ok(fetched_json) => {
                    write_to_cache(&fetched_json, &cache_file_path);
                    Ok(fetched_json)
                },
                Err(e) => Err(e),
            }
        }
    }
}

fn fetch_word_json(settings: &Config, word: &str) -> Result<String, Error> {
    let token = settings.get_str("token").unwrap();
    let word_client = wordsapi_client::WordClient::new(&token);
    let result = word_client.look_up(word);
    match result {
        Ok(wr) => Ok(wr.raw_json().to_string()),
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }
}

fn display_word_data(word_data: WordData, opt: &Opt) {
    /*
    if opt.json {
        display_json(&mut response)
    } else {
    */
    //write_to_cache(&json, &opt);
    display_definition(&mut word_data);
}

fn display_json(response: &mut WordResponse) {
    println!("{}", response.raw_json());
}

fn write_to_cache(json: &str, cache_file_path: &PathBuf) {    
    match fs::File::create(cache_file_path) {
        Ok(cache_file) => write_to_cache_file(&json, cache_file),
        Err(e) => println!("Warning: could not write cache file: {}", e)
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

fn create_cache_dir(ref cache_dir: &PathBuf) {
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

fn get_cache_file_path(ref cache_dir: &PathBuf, word: &str) -> PathBuf {
    let fname = format!("{}.json", word);
    println!("saving using file name: '{}'", fname);
    let fname = cache_dir.join(fname);
    println!("will be located under: '{:?}'", fname);
    fname
}

fn read_cache_file(ref cache_file_path: &PathBuf) -> io::Result<String> {
    let mut cache_file = fs::File::open(cache_file_path)?;
    let mut contents = String::new();
    let _size = cache_file.read_to_string(&mut contents)?;
    Ok(contents)
}