extern crate config;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate wordsapi_client;

use structopt::StructOpt;
use wordsapi_client::{WordData, WordResponse};

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
        let data = response.try_parse();
        match data {
            Ok(mut wd) => display_definition(&mut wd),
            Err(e) => println!("WordAPI Clent response error: {}", e),
        }
    }
}

fn display_json(response: &mut WordResponse) {
    println!("{}", response.raw_json());
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
