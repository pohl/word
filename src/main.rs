extern crate wordsapi_client;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate config;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "word", about = "Look up a word.")]
struct Opt {
    #[structopt(short = "d", long = "debug", help = "Activate debug mode")]
    debug: bool,
    #[structopt(help = "The word to look up")]
    word: String,
    #[structopt(help = "API token, from environment if not present")]
    token: Option<String>,
}

 
fn main() { 
    let opt = Opt::from_args();
    println!("{:?}", opt);

    let mut settings = config::Config::default();
    settings
	.merge(config::File::with_name("Settings")).unwrap()
        .merge(config::Environment::with_prefix("WORD")).unwrap();
    let token = settings.get_str("token").unwrap();
    wordsapi_client::look_up_word(&opt.word, &token);
}


