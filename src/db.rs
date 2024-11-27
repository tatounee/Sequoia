use color_eyre::eyre::Result;
use rusqlite::{config::DbConfig, Connection};
use tracing::{debug, info, instrument};

use crate::client::{Client, Group};
use crate::email::{Email, PlainEmail, TemplateEmail};
use crate::mailer::Mailer;

pub struct DB {
    path: String,
    connection: Connection,
}

impl DB {
    #[instrument(skip_all)]
    pub fn connect() -> Result<Self> {
        let path = dotenvy::var("DB_PATH")?;

        let connection = Connection::open(&path)?;
        info!("Connected to {}", path);

        // Enable foreign keys
        if !connection.db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_FKEY)? {
            debug!("Enable foreign keys");
            connection.set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_FKEY, true)?;
        };

        let this = Self {
            path: path.to_owned(),
            connection,
        };

        this.init()?;

        Ok(this)
    }

    fn init(&self) -> Result<()> {
        let create_tables = [
            Client::CREATE_TABLES,
            Group::CREATE_TABLES,
            PlainEmail::CREATE_TABLES,
            TemplateEmail::CREATE_TABLES,
            Email::CREATE_TABLES,
            Mailer::CREATE_TABLES
        ]
        .join("\n");

        debug!(create_tables);

        self.connection.execute_batch(&create_tables)?;

        Ok(())
    }

    pub(crate) fn connection(&self) -> &Connection {
        &self.connection
    }

    #[cfg(debug_assertions)]
    pub fn clean(&self) -> Result<()> {
        self.connection.execute_batch(
            r"
            DELETE FROM MM_ClientGroupClient WHERE 0=0;
            DELETE FROM Client WHERE 0=0;
            DELETE FROM ClientGroup WHERE 0=0;
            DELETE FROM MM_EmailClient WHERE 0=0;
            DELETE FROM MM_EmailClientGroup WHERE 0=0;
            DELETE FROM Email WHERE 0=0;
            DELETE FROM PlainEmail WHERE 0=0;
            DELETE FROM TemplateEmail WHERE 0=0;
        ",
        )?;

        Ok(())
    }
}
