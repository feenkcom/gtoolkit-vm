extern crate clap;
extern crate octocrab as github;
extern crate question;
extern crate reqwest;
extern crate semver;
extern crate serde;
extern crate serde_derive;
extern crate tokio;
extern crate tokio_util;

use clap::{AppSettings, Clap};
use github::models::repos::Release;
use github::Octocrab;
use question::{Answer, Question};
use reqwest::Url;
use semver::Version;
use std::error::Error;
use std::path::PathBuf;

#[derive(Clap, Clone, Debug)]
#[clap(version = "1.0", author = "feenk gmbh <contact@feenk.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct ReleaseOptions {
    /// An owner of the repository
    #[clap(long, required(true))]
    owner: String,
    /// A repository name
    #[clap(long, required(true))]
    repo: String,
    /// A name of the environment variable with personal GitHub token. The reason we do not accept tokens directly is because thne it would be exposed in the CI log
    #[clap(long)]
    token: Option<String>,
    /// Force a version in form X.Y.Z
    #[clap(long, conflicts_with_all(&["bump-major", "bump-minor", "bump-patch"]))]
    version: Option<String>,
    /// Increase the major version, dropping minor and patch versions to 0
    #[clap(long, conflicts_with_all(&["version", "bump-minor", "bump-patch"]))]
    bump_major: bool,
    /// Increase the minor version, dropping the patch version to 0
    #[clap(long, conflicts_with_all(&["bump-major", "version", "bump-patch"]))]
    bump_minor: bool,
    /// Increase the patch version
    #[clap(long, conflicts_with_all(&["bump-major", "bump-minor", "version"]))]
    bump_patch: bool,
    /// Allow releaser to make decisions without asking
    #[clap(long)]
    auto_accept: bool,
    /// Attach provided assets to the release
    #[clap(long, parse(from_os_str))]
    assets: Option<Vec<PathBuf>>,
}

pub struct ReleaserError {}

fn create_first_time_version(release_options: &ReleaseOptions) -> Version {
    if release_options.bump_major {
        Version::new(1, 0, 0)
    } else if release_options.bump_minor {
        Version::new(0, 1, 0)
    } else {
        Version::new(0, 0, 1)
    }
}

fn create_next_version(current_version: &Version, release_options: &ReleaseOptions) -> Version {
    if release_options.bump_major {
        Version::new(current_version.major + 1, 0, 0)
    } else if release_options.bump_minor {
        Version::new(current_version.major, current_version.minor + 1, 0)
    } else {
        Version::new(
            current_version.major,
            current_version.minor,
            current_version.patch + 1,
        )
    }
}

async fn upload_asset_file(
    file: &PathBuf,
    release: &Release,
    options: &ReleaseOptions,
    octocrab: &Octocrab,
) -> Result<(), Box<dyn Error>> {
    let uploads_url = format!(
        "https://uploads.github.com/repos/{}/{}/releases/{}/assets",
        options.owner.clone(),
        options.repo.clone(),
        release.id
    );

    let base_url = Url::parse(&uploads_url)?;

    let filename = file.file_name().unwrap().to_str().unwrap();
    let mut new_url = base_url.clone();
    new_url.set_query(Some(format!("{}={}", "name", filename).as_str()));

    let file_size = std::fs::metadata(file)?.len();
    let file = tokio::fs::File::open(file).await?;
    let stream = tokio_util::codec::FramedRead::new(file, tokio_util::codec::BytesCodec::new());
    let body = reqwest::Body::wrap_stream(stream);

    let builder = octocrab
        .request_builder(new_url.as_str(), reqwest::Method::POST)
        .header("Content-Type", "application/octet-stream")
        .header("Content-Length", file_size.to_string());

    builder.body(body).send().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let release_options: ReleaseOptions = ReleaseOptions::parse();

    let mut octocrab_builder = Octocrab::builder();
    if let Some(personal_token) = release_options
        .token
        .as_ref()
        .map(|var_name| std::env::var(var_name).map_or(None, |token| Some(token)))
        .map_or(None, |token| token)
    {
        octocrab_builder = octocrab_builder.personal_token(personal_token);
    }
    let octocrab = octocrab_builder.build()?;

    let latest_release = octocrab
        .repos(release_options.owner.clone(), release_options.repo.clone())
        .releases()
        .get_latest()
        .await
        .map_or(None, |release| Some(release));

    let new_version = match &latest_release {
        None => {
            if !release_options.auto_accept {
                let answer =
                    Question::new("Could not find the latest release. Should we create a new one?")
                        .default(Answer::YES)
                        .show_defaults()
                        .confirm();

                if answer != Answer::YES {
                    return Ok(());
                };
            }
            create_first_time_version(&release_options)
        }
        Some(latest_release) => {
            let tag_name = latest_release.tag_name.trim_start_matches('v');
            match Version::parse(&tag_name) {
                Ok(current_version) => create_next_version(&current_version, &release_options),
                Err(error) => {
                    panic!("Could not parse {:?} as a version: {:?}", &tag_name, &error);
                }
            }
        }
    };

    if !release_options.auto_accept {
        let answer = Question::new(&format!(
            "Are you sure you want to release a new version {}?",
            &new_version.to_string()
        ))
        .default(Answer::YES)
        .show_defaults()
        .confirm();

        if answer != Answer::YES {
            return Ok(());
        };
    }

    let new_release = octocrab
        .repos(release_options.owner.clone(), release_options.repo.clone())
        .releases()
        .create(&format!("v{}", &new_version.to_string()))
        .name(&format!("Release v{}", &new_version.to_string()))
        .send()
        .await?;

    println!(
        "A new release version {:?} published!",
        &new_version.to_string()
    );

    if let Some(assets) = release_options.assets.as_ref() {
        for asset in assets {
            if !release_options.auto_accept {
                let answer =
                    Question::new(&format!("Should asset be uploaded {}?", asset.display()))
                        .default(Answer::YES)
                        .show_defaults()
                        .confirm();

                if answer != Answer::YES {
                    return Ok(());
                };
            }

            upload_asset_file(asset, &new_release, &release_options, &octocrab).await?;
            println!("Should {} be uploaded as a release asset?", asset.display());
        }
    }

    Ok(())
}
