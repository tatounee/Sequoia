use color_eyre::eyre::Result;
use cuid2::create_id;
use serde_derive::{Deserialize, Serialize};
use serde_rusqlite::{columns_from_statement, from_row_with_columns, to_params_named};

use crate::db::DB;

use super::Client;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Group {
    id: String,
    name: String,
    #[serde(skip)]
    clients: Option<Vec<Client>>,
}

impl Group {
    pub(crate) const CREATE_TABLES: &'static str = r#"
        CREATE TABLE IF NOT EXISTS ClientGroup (
            ID    TEXT PRIMARY KEY,
            name  TEXT UNIQUE NOT NULL
        ) STRICT;
        
        CREATE TABLE IF NOT EXISTS MM_ClientGroupClient (
            client_group  TEXT,
            client        TEXT,
            FOREIGN KEY(client_group)  REFERENCES ClientGroup(ID)
                ON UPDATE CASCADE
                ON DELETE CASCADE,
            FOREIGN KEY(client)        REFERENCES Client(ID)
                ON UPDATE CASCADE
                ON DELETE CASCADE
        ) STRICT;
    "#;

    fn new(name: String) -> Self {
        Self {
            id: create_id(),
            name,
            clients: Some(Vec::new()),
        }
    }

    pub fn clients(&self) -> Option<&[Client]> {
        self.clients.as_deref()
    }

    pub fn fetch_client(&mut self, db: &DB) -> Result<()> {
        let mut stmt = db.connection().prepare_cached(
            r"
            SELECT Client.ID, Client.adresse FROM Client 
                LEFT JOIN MM_ClientGroupClient ON MM_ClientGroupClient.client = Client.ID
                WHERE MM_ClientGroupClient.client_group = ?",
        )?;

        let columns = columns_from_statement(&stmt);

        let clients = Result::from_iter(stmt.query_and_then([self.id.clone()], |row| {
            from_row_with_columns::<Client>(row, &columns)
        })?)?;

        self.clients = Some(clients);

        Ok(())
    }

    pub fn create(name: String, db: &DB) -> Result<Self> {
        let this = Self::new(name);

        let mut stmt = db
            .connection()
            .prepare_cached("INSERT INTO ClientGroup (ID, name) VALUES (:id, :name)")?;

        stmt.execute(to_params_named(this.clone_())?.to_slice().as_slice())?;

        Ok(this)
    }

    pub fn get(id: String, db: &DB) -> Result<Option<Self>> {
        let mut stmt = db
            .connection()
            .prepare_cached("SELECT * FROM ClientGroup WHERE ID = ?")?;

        let columns = columns_from_statement(&stmt);

        let mut rows = stmt
            .query_and_then([id], |row| from_row_with_columns::<Self>(row, &columns))
            .unwrap();

        Ok(rows.next().transpose()?)
    }

    pub fn add_client(&mut self, id: String, db: &DB) -> Result<()> {
        let mut stmt = db.connection().prepare_cached(
            "INSERT INTO MM_ClientGroupClient (client_group, client) VALUES (?, ?)",
        )?;

        stmt.execute((self.id.clone(), id))?;

        Ok(())
    }

    pub fn add_clients(&mut self, ids: &[String], db: &DB) -> Result<()> {
        let mut stmt = db.connection().prepare_cached(
            "INSERT INTO MM_ClientGroupClient (client_group, client) VALUES (?, ?)",
        )?;

        for id in ids {
            stmt.execute((self.id.clone(), id))?;
        }

        Ok(())
    }

    pub fn remove_client(&mut self, id: String, db: &DB) -> Result<()> {
        let mut stmt = db.connection().prepare_cached(
            "DELETE FROM MM_ClientGroupClient WHERE client_group = ? AND client = ?",
        )?;

        stmt.execute((self.id.clone(), id))?;

        Ok(())
    }

    pub fn remove_clients(&mut self, ids: &[String], db: &DB) -> Result<()> {
        let mut stmt = db.connection().prepare_cached(
            "DELETE FROM MM_ClientGroupClient WHERE client_group = ? AND client = ?",
        )?;

        for id in ids {
            stmt.execute((self.id.clone(), id))?;
        }

        Ok(())
    }
}

impl Group {
    fn clone_(&self) -> Self {
        let clients = self
            .clients
            .as_ref()
            .map(|clients| clients.iter().map(Client::clone_).collect());

        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            clients,
        }
    }
}
