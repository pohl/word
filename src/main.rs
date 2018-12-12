extern crate config;
extern crate dirs;
extern crate itertools;
extern crate serde_json;
extern crate structopt;
extern crate wordsapi_client;

use config::Config;
use std::fs;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::{Read, Write};
use std::path::PathBuf;
use structopt::StructOpt;
use wordsapi_client::{WordAPIError, WordData, WordEntry, WordRequestType};

#[derive(StructOpt, Debug)]
#[structopt(name = "word", about = "Look up a word.")]
struct Opt {
    #[structopt(short = "a", long = "antonym", help = "Show antonyms for the word")]
    antonym: bool,
    #[structopt(short = "s", long = "synonym", help = "Show synonyms for the word")]
    synonym: bool,
    #[structopt(short = "e", long = "hypernym", help = "Show hypernyms for the word")]
    hypernym: bool,
    #[structopt(short = "o", long = "hyponym", help = "Show hyponyms for the word")]
    hyponym: bool,
    #[structopt(short = "l", long = "holonym", help = "Show holonyms for the word")]
    holonym: bool,
    #[structopt(short = "A", long = "all", help = "Show all the nyms")]
    all: bool,
    #[structopt(short = "j", long = "json", help = "Output raw json")]
    json: bool,
    #[structopt(short = "v", long = "verbose", help = "Show verbose output")]
    verbose: bool,
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
            Ok(word_data) => {
                let word_display = WordDisplay::new(word_data, opt);
                word_display.display_word_data();
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
    let result = word_client.look_up(&opt.word, &WordRequestType::Everything);
    match result {
        Ok(wr) => {
            if opt.verbose {
                println!(
                    "{} API requests remaining of {}.",
                    &wr.rate_limit_remaining, &wr.rate_limit_requests_limit
                );
            }
            Ok(wr.response_json)
        }
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }
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

struct WordDisplay<'a> {
    data: WordData,
    options: &'a Opt,
}

impl<'a> WordDisplay<'a> {
    pub fn new(data: WordData, options: &'a Opt) -> WordDisplay<'a> {
        WordDisplay { data, options }
    }

    fn display_word_data(&self) {
        self.display_variants();
    }

    fn display_variants(&self) {
        self.display_pronunciation();
        for e in &self.data.results {
            self.display_variant(e);
        }
    }

    fn display_pronunciation(&self) {
        if let Some(ref pronunciations) = self.data.pronunciation {
            let pronunciation = match pronunciations.get("all") {
                Some(p) => p,
                None => "",
            };
            println!("{} |{}|", self.data.word, pronunciation);
        }
    }

    fn display_variant(&self, entry: &WordEntry) {
        println!(
            "({}) {}",
            entry.part_of_speech.as_ref().unwrap_or(&"?".to_string()),
            entry.definition
        );
        if self.options.antonym || self.options.all {
            self.display_nyms(&entry.antonyms, "antonyms");
        }
        if self.options.synonym || self.options.all {
            self.display_nyms(&entry.synonyms, "synonyms");
        }
        if self.options.hypernym || self.options.all {
            self.display_nyms(&entry.type_of, "hypernyms");
        }
        if self.options.hyponym || self.options.all {
            self.display_nyms(&entry.has_types, "hyponyms");
        }
        if self.options.holonym || self.options.all {
            self.display_nyms(&entry.part_of, "holonyms");
        }
        println!();
    }

    fn display_nyms(&self, nyms: &Option<Vec<String>>, label: &str) {
        let result = match *nyms {
            Some(ref ns) => ns.join(","),
            None => "(None)".to_owned(),
        };
        println!("   {}: {}", label, result);
    }
}

fn create_cache_dir(cache_dir: &PathBuf) {
    match fs::create_dir_all(&cache_dir) {
        Ok(_) => (),
        Err(e) => {
            println!("Warning: could not create cache directory: {}", e)
        }
    }
}

fn get_cache_dir() -> PathBuf {
    match dirs::home_dir() {
        Some(path) => path.join(".word"),
        None => PathBuf::from("./.word"),
    }
}

fn get_cache_file_path(cache_dir: &PathBuf, opt: &Opt) -> PathBuf {
    let stem: String = opt
        .word
        .chars()
        .map(|x| match x {
            ' ' => '_',
            _ => x,
        })
        .collect();
    let filename = format!("{}.json", stem);
    if opt.verbose {
        println!("saving using file name: '{}'", filename);
    }
    let filename = cache_dir.join(filename);
    if opt.verbose {
        println!("will be located under: '{:?}'", filename);
    }
    filename
}

fn read_cache_file(cache_file_path: &PathBuf) -> io::Result<String> {
    let mut cache_file = fs::File::open(cache_file_path)?;
    let mut contents = String::new();
    let _size = cache_file.read_to_string(&mut contents)?;
    Ok(contents)
}
