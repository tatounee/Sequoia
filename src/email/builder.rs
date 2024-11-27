use color_eyre::eyre::Result;
use email_address::EmailAddress;
use tracing::error;

use crate::{db::DB, email::TemplateEmail};

use super::{Email, EmailModel, PlainEmail};

#[derive(Default)]
pub struct EmailBuilder {
    subject: Option<String>,
    plain_body: Option<String>,
    template_body: Option<String>,
    sender_adresse: Option<EmailAddress>,
    source_path: Option<String>,
}

impl EmailBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subject(mut self, subject: &str) -> Self {
        self.subject = Some(subject.to_owned());
        self
    }

    pub fn sender_adresse(mut self, sender_adresse: &str) -> Result<Self> {
        self.sender_adresse = Some(sender_adresse.parse()?);
        Ok(self)
    }

    pub fn plain_body(mut self, body: &str) -> Self {
        if self.template_body.is_some() {
            error!("Plain body and template body are incompatible (template body already set)");
            panic!();
        }

        self.plain_body = Some(body.to_owned());
        self
    }

    pub fn template_body(mut self, body: &str) -> Self {
        if self.plain_body.is_some() {
            error!("Plain body and template body are incompatible (plain body already set)");
            panic!();
        }

        self.template_body = Some(body.to_owned());
        self
    }

    pub fn source_path(mut self, source_path: &str) -> Self {
        self.source_path = Some(source_path.to_owned());
        self
    }

    pub fn create(self, db: &DB) -> Result<Email> {
        let subject = self.subject.unwrap_or_default();

        let sender_adresse = if let Some(sender_adresse) = self.sender_adresse {
            sender_adresse
        } else {
            error!("Sender adresse is required when building email");
            panic!();
        };

        let email = if let Some(plain_body) = self.plain_body {
            EmailModel::Plain(PlainEmail::new(subject, plain_body))
        } else if let Some(template_body) = self.template_body {
            let source_path = if let Some(source_path) = self.source_path {
                source_path
            } else {
                error!("Source path is required when building template email");
                panic!();
            };

            EmailModel::Template(TemplateEmail::new(subject, template_body, source_path))
        } else {
            EmailModel::Plain(PlainEmail::new(subject, "".to_owned()))
        };

        Email::create(sender_adresse, email, db)
    }
}
