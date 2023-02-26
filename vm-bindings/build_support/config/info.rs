use anyhow::anyhow;
use chrono::{DateTime, FixedOffset, TimeZone};
use git2::{DescribeFormatOptions, DescribeOptions, Repository};
use semver::Version;
use serde::Serialize;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

use crate::Builder;

#[derive(Debug, Clone, Serialize)]
pub struct BuildInfo {
    long_hash: String,
    short_hash: String,
    #[serde(skip)]
    author_date: DateTime<FixedOffset>,
    tag: Option<String>,
    version: Option<Version>,
}

impl BuildInfo {
    pub fn new(builder: Rc<dyn Builder>) -> anyhow::Result<Self> {
        let repo = Repository::open(builder.vm_sources_directory())?;

        let ref_head = repo.find_reference("HEAD")?;
        let commit = ref_head.peel_to_commit()?;

        let time = commit.time();
        let author_date = if time.sign() == '-' {
            FixedOffset::west_opt(time.offset_minutes() * 60)
                .and_then(|offset| offset.timestamp_opt(time.seconds(), 0).single())
                .unwrap()
        } else {
            FixedOffset::east_opt(time.offset_minutes() * 60)
                .and_then(|offset| offset.timestamp_opt(time.seconds(), 0).single())
                .unwrap()
        };

        let commit_object = commit.into_object();
        let long_hash = commit_object.id().to_string();
        let short_hash = commit_object
            .short_id()?
            .as_str()
            .map(str::to_string)
            .ok_or(anyhow!("Failed to get the short commit hash"))?;

        let mut opts = DescribeOptions::new();
        let _ = opts.describe_tags();
        let mut format_opts = DescribeFormatOptions::new();
        let _ = format_opts.dirty_suffix("-dirty");

        let tag = repo
            .describe(&opts)
            .map_or_else(|_| None, |x| x.format(Some(&format_opts)).ok());

        let version = tag
            .as_ref()
            .map(|tag| tag.strip_prefix("v").unwrap_or(tag))
            .map(|tag| Version::parse(tag))
            .map_or(None, |result| result.ok());

        Ok(Self {
            long_hash,
            short_hash,
            author_date,
            tag,
            version,
        })
    }

    pub fn short_hash(&self) -> &str {
        self.short_hash.as_str()
    }

    pub fn author_date(&self) -> &DateTime<FixedOffset> {
        &self.author_date
    }

    pub fn tag(&self) -> Option<&str> {
        self.tag.as_ref().map(String::as_str)
    }

    pub fn version(&self) -> Option<&Version> {
        self.version.as_ref()
    }

    pub fn version_major(&self) -> usize {
        self.version().map_or(0, |version| version.major as usize)
    }

    pub fn version_minor(&self) -> usize {
        self.version().map_or(0, |version| version.minor as usize)
    }

    pub fn version_patch(&self) -> usize {
        self.version().map_or(1, |version| version.patch as usize)
    }
}

impl Display for BuildInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(tag) = self.tag() {
            write!(f, "{} - ", tag)?;
        };

        write!(f, "Commit: {}", self.short_hash())?;
        write!(f, " - ")?;
        write!(f, "Date: {}", self.author_date().to_string())?;

        Ok(())
    }
}
