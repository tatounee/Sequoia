use color_eyre::eyre::{Context, Result};
use cuid2::create_id;
use email_address::EmailAddress;

mod builder;
mod plain_email;
mod tags;
mod template_email;

pub use builder::EmailBuilder;
pub use plain_email::PlainEmail;
use rusqlite::types::Null;
use serde_derive::{Deserialize, Serialize};
use serde_rusqlite::{columns_from_statement, from_row_with_columns};
use tags::Tags;
pub use template_email::TemplateEmail;
use tracing::{error, info, instrument};

use crate::db::DB;

#[derive(Debug)]
pub struct Email {
    id: String,
    sender_adresse: EmailAddress,
    tags: Tags,
    email: EmailModel,
}

impl Email {
    pub(crate) const CREATE_TABLES: &'static str = r#"
        CREATE TABLE IF NOT EXISTS Email (
            ID                  TEXT PRIMARY KEY,
            sender_adresse      TEXT,
            tags                TEXT,
            email_discriminant  INTEGER CHECK(email_discriminant IN (0, 1)),
            plain_email_ID      TEXT,
            template_email_ID   TEXT,
            FOREIGN KEY (plain_email_ID)     REFERENCES PlainEmail(ID)
                ON UPDATE CASCADE
                ON DELETE CASCADE,
            FOREIGN KEY (template_email_ID)  REFERENCES TemplateEmail(ID)
                ON UPDATE CASCADE
                ON DELETE CASCADE
        ) STRICT;
        "#;

    fn new(sender_adresse: EmailAddress, email: EmailModel, tags: Tags) -> Self {
        Self {
            id: create_id(),
            sender_adresse,
            tags,
            email,
        }
    }

    pub fn builder() -> EmailBuilder {
        EmailBuilder::new()
    }

    pub async fn create(
        sender_adresse: EmailAddress,
        email: EmailModel,
        tags: Vec<String>,
        db: &DB,
    ) -> Result<Self> {
        let this = Self::new(sender_adresse, email, tags.try_into()?);

        this.email.write(db).await?;

        db.connection(|conn| {
            let mut stmt = conn.prepare(r"
                    INSERT INTO Email (ID, sender_adresse, tags, email_discriminant, plain_email_ID, template_email_ID) 
                    VALUES (?, ?, ?, ?, ?, ?)
                ")?;
    
            match &this.email {
                EmailModel::Plain(plain_email) => {
                    stmt.execute((
                        &this.id,
                        this.sender_adresse.to_string(),
                        &this.tags.to_string(),
                        0,
                        plain_email.id(),
                        Null,
                    ))?;
                }
                EmailModel::Template(template_email) => {
                    stmt.execute((
                        &this.id,
                        this.sender_adresse.to_string(),
                        1,
                        Null,
                        template_email.id(),
                    ))?;
                }
            }
            
            Ok(())
        }).await?;

        Ok(this)
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub fn sender_adresse(&self) -> &str {
        self.sender_adresse.as_ref()
    }

    pub fn subject(&self) -> &str {
        match &self.email {
            EmailModel::Plain(plain_email) => plain_email.subject(),
            EmailModel::Template(_) => unimplemented!("Template email aren't yet supported"),
        }
    }

    pub fn body(&self) -> String {
        match &self.email {
            EmailModel::Plain(plain_email) => plain_email.body().to_owned(),
            EmailModel::Template(_) => unimplemented!("Template email aren't yet supported"),
        }
    }

    pub async fn get_one(id: &str, db: &DB) -> Result<Option<Self>> {
        db.connection(|conn| {
            let mut stmt = conn.prepare_cached(
                r"
                SELECT em.ID, em.tags, em.sender_adresse, em.email_discriminant,
                  pe.ID as plain_email_id, pe.subject as plain_subject, pe.body as plain_body,
                  te.ID as template_email_id, te.subject as template_subject, te.body as template_body, te.source_path as template_source_path
                    FROM Email em
                    LEFT JOIN PlainEmail pe ON em.plain_email_ID = pe.ID
                    LEFT JOIN TemplateEmail te ON em.template_email_ID = te.ID
                    WHERE em.ID = ?
            ",
            )?;
    
            let columns = columns_from_statement(&stmt);
    
            info!("{columns:?}");
    
            let mut rows =
                stmt.query_and_then([id], |row| from_row_with_columns::<SQLEmail>(row, &columns))?;
    
            rows.next().transpose()?.map(Email::try_from).transpose()
        }).await

    }

    #[cfg(debug_assertions)]
    pub fn id_(&self) -> &str {
        &self.id
    }
}

#[derive(Debug)]
pub enum EmailModel {
    /// SQL discriminant : **0**.
    Plain(PlainEmail),
    /// SQL discriminant : **1**.
    Template(TemplateEmail),
}

impl EmailModel {
    async fn write(&self, db: &DB) -> Result<()> {
        match self {
            Self::Plain(plain_email) => plain_email.write(db).await?,
            Self::Template(template_email) => template_email.write(db).await?,
        }

        Ok(())
    }
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
struct SQLEmail {
    ID: String,
    sender_adresse: String,
    tags: String,
    email_discriminant: u8,
    plain_email_id: Option<String>,
    plain_subject: Option<String>,
    plain_body: Option<String>,
    template_email_id: Option<String>,
    template_subject: Option<String>,
    template_body: Option<String>,
    template_source_path: Option<String>,
}

impl TryFrom<SQLEmail> for Email {
    type Error = color_eyre::eyre::Error;

    #[instrument]
    fn try_from(value: SQLEmail) -> Result<Self> {
        // TODO: Gérer les erreurs correctement

        let email_model = match value.email_discriminant {
            0 => {
                let plain_email = PlainEmail::from_sql(
                    value.plain_email_id.unwrap(),
                    value.plain_subject.unwrap(),
                    value.plain_body.unwrap(),
                );
                EmailModel::Plain(plain_email)
            }
            1 => {
                unimplemented!("Template email are not yet implemented")
            }
            _ => {
                error!("Unknown email model discriminant.");
                panic!();
            }
        };

        info!("");

        Ok(Self {
            id: value.ID,
            tags: Tags::from(value.tags),
            sender_adresse: dbg!(value
                .sender_adresse)
                .parse()
                .context("Parsing sender_adresse adresse from database")
                .unwrap(),
            email: email_model,
        })
    }
}
