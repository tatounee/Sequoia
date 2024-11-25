use cuid2::create_id;
use email_address::EmailAddress;

mod plain_email;
mod template_email;
mod builder;

pub use plain_email::PlainEmail;
pub use template_email::TemplateEmail;
pub use builder::EmailBuilder;

pub struct Email {
    id: String,
    sender_adresse: EmailAddress,
    email: EmailModel,
    // TODO: tag: Vec<String>,
}

impl Email {
    pub(crate) const CREATE_TABLES: &'static str = r#"
    CREATE TABLE IF NOT EXISTS Email (
        ID                  TEXT PRIMARY KEY,
            sender_adresse      TEXT,
            email_discriminant  TEXT CHECK(email_discriminant IN (0, 1)),
            plain_email         TEXT,
            template_email      TEXT,
            FOREIGN KEY (plain_email)     REFERENCES PlainEmail(ID),
            FOREIGN KEY (template_email)  REFERENCES TemplateEmail(ID)
        );
        "#;

    pub(crate) fn new(sender_adresse: EmailAddress, email: EmailModel) -> Self {
        Self {
            id: create_id(),
            sender_adresse,
            email,
        }
    }
    
    pub fn sender_adresse(&self) -> &str {
        self.sender_adresse.as_ref()
    }

    pub fn subject(&self) -> String {
        match &self.email {
            EmailModel::Plain(plain_email) => plain_email.subject().to_owned(),
            EmailModel::Template(_) => unimplemented!("Template email aren't yet supported")
        }
    }

    pub fn body(&self) -> String {
        match &self.email {
            EmailModel::Plain(plain_email) => plain_email.body().to_owned(),
            EmailModel::Template(_) => unimplemented!("Template email aren't yet supported")
        }
    }
}

#[repr(u8)]
pub enum EmailModel {
    Plain(PlainEmail) = 0,
    Template(TemplateEmail) = 1,
}
