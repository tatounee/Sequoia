use std::str::FromStr;

use color_eyre::eyre::Result;
use cuid2::create_id;
use email_address::EmailAddress;
use serde_derive::{Deserialize, Serialize};

mod group;
mod client_ref;

pub use group::Group;
use serde_rusqlite::{columns_from_statement, from_row_with_columns, to_params_named};

use crate::{db::DB, email::Email};

#[derive(Serialize, Deserialize, Debug)]
pub struct Client {
    #[serde(rename(deserialize = "ID"))]
    id: String,
    adresse: EmailAddress,
    #[serde(skip)]
    received_emails: Option<Vec<Email>>,
}

impl Client {
    pub(crate) const CREATE_TABLES: &'static str = r#"
        CREATE TABLE IF NOT EXISTS Client (
            ID       TEXT PRIMARY KEY,
            adresse  TEXT NOT NULL
            ) STRICT;
    "#;

    fn new(adresse: &str) -> Result<Self> {
        let adresse = EmailAddress::from_str(adresse)?;

        Ok(Self {
            id: create_id(),
            adresse,
            received_emails: None,
        })
    }

    pub async fn create(adresse: &str, db: &DB) -> Result<Self> {
        let this = Self::new(adresse)?;

        db.connection(|conn| {
            let mut stmt =
                conn.prepare_cached("INSERT INTO Client (ID, adresse) VALUES (:id, :adresse)")?;

            stmt.execute(to_params_named(&this)?.to_slice().as_slice())?;

            Ok(())
        })
        .await?;

        Ok(this)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn adresse(&self) -> &str {
        self.adresse.as_ref()
    }

    pub async fn get_one(id: String, db: &DB) -> Result<Option<Self>> {
        db.connection(|conn| {
            let mut stmt = conn.prepare_cached("SELECT * FROM Client WHERE ID = ?")?;

            let columns = columns_from_statement(&stmt);

            let mut rows =
                stmt.query_and_then([id], |row| from_row_with_columns::<Self>(row, &columns))?;

            Ok(rows.next().transpose()?)
        })
        .await
    }

    pub async fn get_many<I: Iterator<Item = String>>(
        ids: I,
        db: &DB,
    ) -> Result<Vec<Option<Self>>> {
        db.connection(|conn| {
            let mut stmt = conn.prepare_cached("SELECT * FROM Client WHERE ID = ?")?;

            let columns = columns_from_statement(&stmt);

            Ok(Result::from_iter(ids.map(|id| {
                let mut rows = stmt
                    .query_and_then([id], |row| from_row_with_columns::<Self>(row, &columns))
                    .unwrap();

                rows.next().transpose()
            }))?)
        })
        .await
    }
}
