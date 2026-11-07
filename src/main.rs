use std::path::{Path, PathBuf};

use clap::Parser;
use futures_util::StreamExt;
use tokio::{io::AsyncWriteExt, try_join};
use unic_langid::LanguageIdentifier;

use anyhow::Result;

mod input;
mod model;

mod musl_parse;

#[derive(Parser)]
#[command(name = "cldr-musl-extractor")]
#[command(about = "Obtain musl-locales directly from CLDR")]
pub struct Args {
    #[arg(required = true)]
    targets: Vec<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    println!("Checking CLDR and musl repos");

    if let Err(error) = try_join!(
        get_repository(
            "https://github.com/unicode-org/cldr/archive/refs/heads/master.zip",
            "cldr",
            Path::new("./")
        ),
        get_repository(
            "https://codeberg.org/musl/musl-locales/refs/heads/master.zip",
            "musl-locales",
            Path::new("./")
        )
    ) {
        eprintln!("Failed to get repositories: {error:?}");
        return;
    }

    if let Some("all") = args.targets.first().map(|s| s.as_str()) {
        for locale in ALL_LOCALES {
            work_locale(locale).await;
        }
    } else {
        for target in &args.targets {
            work_locale(target).await;
        }
    }
}

async fn work_locale_inner(locale_str: &str) -> Result<()> {
    let parsed_langid: LanguageIdentifier = locale_str
        .parse()
        .expect("Failed to parse input as language identifier");

    let lang = parsed_langid.language.as_str();

    let region = parsed_langid
        .region
        .map(|r| r.as_str().to_string())
        .unwrap_or("".to_string());

    let messages = musl_parse::parse_messages(lang, &region).await?;

    let locale = input::get_locale_data(lang, &region, messages).await?;

    locale.to_file(Path::new(&format!("{}_{}", lang, &region)))?;

    Ok(())
}

async fn work_locale(locale_str: &str) {
    if let Err(e) = work_locale_inner(locale_str).await {
        eprintln!("Locale {locale_str} failed: {e:?}")
    }
}

async fn get_repository(url: &str, tmp_name: &str, dest: &Path) -> Result<()> {
    if dest.exists() {
        return Ok(());
    }

    let response = reqwest::get(url).await?.error_for_status()?;
    let zip_file = PathBuf::from(format!("/tmp/cldr_musl_extractor/{tmp_name}.zip"));
    let mut file = tokio::fs::File::create(&zip_file).await?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;

        file.write_all(&chunk).await?;
    }

    file.flush().await?;

    unzip_async(&zip_file, dest).await?;

    tokio::fs::remove_file(zip_file).await?;

    Ok(())
}

pub async fn unzip_async(target: &Path, dest: &Path) -> Result<()> {
    let target = target.to_owned();
    let dest = dest.to_owned();

    tokio::task::spawn_blocking(move || unzip(&target, &dest)).await?
}

pub fn unzip(target: &Path, dest: &Path) -> Result<()> {
    let mut archive = zip::ZipArchive::new(std::fs::File::open(target)?)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;

        // GH encloses everything in <repo-branch>/file so we are skipping this
        let relative_path = entry
            .enclosed_name()
            .ok_or_else(|| anyhow::anyhow!("invalid path in zip"))?
            .components()
            .skip(1)
            .collect::<std::path::PathBuf>();

        let out_path = dest.join(relative_path);

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(&parent)?;
            }

            let mut out_file = std::fs::File::create(&out_path)?;

            std::io::copy(&mut entry, &mut out_file)?;
        }
    }

    Ok(())
}

const ALL_LOCALES: &[&str] = &["en_US", "es_ES"];
