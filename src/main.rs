use std::path::Path;

use clap::Parser;

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

    // TODO: Extract region and language
    let locale = input::get_locale_data("", "")
        .await
        .expect("Failed to get locale data");

    locale
        .to_file(Path::new(&args.target))
        .expect("Failed to write locale file");
}
