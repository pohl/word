extern crate wordsapi_client;
extern crate clap; 
use clap::{Arg, App}; 
 
fn main() { 
    let matches = App::new("word")
       .version("1.0")
       .about("A simple client for WordsAPI.")
       .author("Pohl Longsine")
       .arg(Arg::with_name("WORD")
            .help("Sets the input file to use")
            .required(true)
            .index(1))
       .get_matches(); 
    let words: Vec<&str> = matches.values_of("WORD").unwrap().collect();
    for word in &words {
        wordsapi_client::look_up_word(&word)
    }
}


