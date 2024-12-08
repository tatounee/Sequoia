use std::fmt::Display;

use color_eyre::eyre::Result;
use tracing::warn;

const TAG_SEPARATOR: char = '$';

/// A list of tag (`String`). '$' is forbidden in a tag.
#[derive(Debug, Default)]
pub(super) struct Tags {
    tags: Vec<String>,
}

impl Tags {
    pub(super) fn new(tags: Vec<String>) -> Result<Self> {
        // TODO: Gérer les erreurs correctement

        if let Some(tag) = tags.iter().find(|tag| tag.contains(TAG_SEPARATOR)) {
            warn!("{TAG_SEPARATOR} is forbidden in tag. Found it in {tag}");
            panic!()
        }

        Ok(Self { tags })
    }

    pub(super) fn push(&mut self, tag: String) -> Result<(), String> {
        if tag.contains(TAG_SEPARATOR) {
            Err(tag)
        } else {
            self.tags.push(tag);
            Ok(())
        }
    }

    pub(super) fn remove(&mut self, tag: &str) -> bool {
        if let Some(idx) = self.tags.iter().position(|tag_| tag_.as_str() == tag) {
            self.tags.remove(idx);
            true
        } else {
            false
        }
    }

    pub(super) fn into_vec(self) -> Vec<String> {
        self.tags
    }
}

impl Display for Tags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.tags.join(&TAG_SEPARATOR.to_string()))
    }
}

impl TryFrom<Vec<String>> for Tags {
    type Error = color_eyre::eyre::Error;

    fn try_from(value: Vec<String>) -> Result<Self> {
        Self::new(value)
    }
}

impl From<String> for Tags {
    fn from(value: String) -> Self {
        let tags = value.split(TAG_SEPARATOR).map(str::to_owned).collect();

        Self { tags }
    }
}
