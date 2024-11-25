use std::str::FromStr;

use color_eyre::eyre::Result;
use cuid2::create_id;
use email_address::EmailAddress;

use super::{Email, EmailModel};

pub struct PlainEmail {
    id: String,
    subject: String,
    body: String,
}

impl PlainEmail {
    pub(crate) const CREATE_TABLES: &'static str = r#"
    CREATE TABLE IF NOT EXISTS PlainEmail (
        ID           TEXT PRIMARY KEY,
        subject      TEXT,
        body         TEXT
        ) STRICT;
        "#;

    pub fn new(subject: String, body: String) -> Self {
        Self {
            id: create_id(),
            subject,
            body,
        }
    }

    pub fn into_email(self, sender_adresse: &str) -> Result<Email> {
        let sender_adresse = EmailAddress::from_str(sender_adresse)?;
        let email = EmailModel::Plain(self);

        Ok(Email::new(sender_adresse, email))
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn body(&self) -> &str {
        &self.body
    }
}
