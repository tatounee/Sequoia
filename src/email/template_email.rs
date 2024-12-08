use color_eyre::eyre::Result;
use cuid2::create_id;
use serde_derive::{Deserialize, Serialize};
use serde_rusqlite::to_params_named;

use crate::db::DB;

#[derive(Deserialize, Serialize, Debug)]
pub struct TemplateEmail {
    #[serde(rename(deserialize = "ID"))]
    id: String,
    subject: String,
    body: String,
    source_path: String,
}

impl TemplateEmail {
    pub(crate) const CREATE_TABLES: &'static str = r#"
        CREATE TABLE IF NOT EXISTS TemplateEmail (
            ID           TEXT PRIMARY KEY,
            subject      TEXT,
            body         TEXT,
            source_path  TEXT
        ) STRICT;
    "#;

    pub(super) fn new(subject: String, body: String, source_path: String) -> Self {
        Self {
            id: create_id(),
            subject,
            body,
            source_path,
        }
    }

    pub async fn create(
        subject: String,
        body: String,
        source_path: String,
        db: &DB,
    ) -> Result<Self> {
        let this = Self::new(subject, body, source_path);

        db.connection(|conn| {
            let mut stmt = conn.prepare_cached(
                "INSERT INTO TemplateEmail (ID, subject, body, source_path) VALUES (:id, :adresse, :body, :source_path)",
            )?;

            stmt.execute(to_params_named(&this)?.to_slice().as_slice())?;

            Ok(())
        }).await?;

        Ok(this)
    }

    pub(super) async fn write(&self, db: &DB) -> Result<()> {
        db.connection(|conn| {
            let mut stmt = conn.prepare_cached(
                "INSERT INTO TemplateEmail (ID, subject, body, source_path) VALUES (:id, :adresse, :body, :source_path)",
            )?;

            stmt.execute(to_params_named(self)?.to_slice().as_slice())?;

            Ok(())
        }).await
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}
