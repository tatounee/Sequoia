// use std::str::FromStr;

// use color_eyre::eyre::Result;
// use email_address::EmailAddress;

use color_eyre::eyre::Result;
// use super::{Email, EmailModel};
use cuid2::create_id;
use serde_derive::{Deserialize, Serialize};
use serde_rusqlite::to_params_named;

use crate::db::DB;

#[derive(Deserialize, Serialize, Debug)]
pub struct PlainEmail {
    #[serde(rename(deserialize = "ID"))]
    id: String,
    subject: String,
    body: String,
}

impl PlainEmail {
    pub(crate) const CREATE_TABLES: &'static str = r#"
        CREATE TABLE IF NOT EXISTS PlainEmail (
            ID       TEXT PRIMARY KEY,
            subject  TEXT,
            body     TEXT
        ) STRICT;
        "#;

    pub(super) fn new(subject: String, body: String) -> Self {
        Self {
            id: create_id(),
            subject,
            body,
        }
    }

    pub(super) fn from_sql(id: String, subject: String, body: String) -> Self {
        Self { id, subject, body }
    }

    pub fn create(subject: String, body: String, db: &DB) -> Result<Self> {
        let this = Self::new(subject, body);

        let mut stmt = db.connection().prepare_cached(
            "INSERT INTO PlainEmail (ID, subject, body) VALUES (:id, :subject, :body)",
        )?;

        stmt.execute(to_params_named(&this)?.to_slice().as_slice())?;

        Ok(this)
    }

    pub(super) fn write(&self, db: &DB) -> Result<()> {
        let mut stmt = db.connection().prepare_cached(
            "INSERT INTO PlainEmail (ID, subject, body) VALUES (:id, :subject, :body)",
        )?;

        stmt.execute(to_params_named(self)?.to_slice().as_slice())?;

        Ok(())
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}
