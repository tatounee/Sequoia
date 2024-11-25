use cuid2::create_id;

pub struct TemplateEmail {
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

    pub fn new(subject: String, body: String, source_path: String) -> Self {
        Self {
            id: create_id(),
            subject,
            body,
            source_path,
        }
    }
}
