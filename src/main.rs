use std::path::Path;

use clap::Parser;
use unic_langid::LanguageIdentifier;

mod input;
mod model;
mod output;

#[derive(Parser)]
#[command(name = "cldr-musl-extractor")]
#[command(about = "Obtain musl-locales directly from CLDR")]
pub struct Args {
    target: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    let parsed_langid: LanguageIdentifier = args
        .target
        .parse()
        .expect("Failed to parse input as language identifier");

    let region = parsed_langid
        .region
        .map(|r| r.as_str().to_string())
        .unwrap_or("".to_string());

    let locale = input::get_locale_data(parsed_langid.language.as_str(), &region)
        .await
        .expect("Failed to get locale data");

    locale
        .to_file(Path::new(&args.target))
        .expect("Failed to write locale file");
}
