use color_eyre::eyre::Result;
use cuid2::create_id;
use serde_derive::{Deserialize, Serialize};
use serde_rusqlite::{columns_from_statement, from_row_with_columns, to_params_named};

use crate::db::DB;

use super::{client_ref::ClientRef, Client};

#[derive(Serialize, Deserialize, Debug)]
pub struct Group {
    #[serde(rename(deserialize = "ID"))]
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
            client_group_ID  TEXT,
            client_ID        TEXT,
            FOREIGN KEY(client_group_ID)  REFERENCES ClientGroup(ID)
                ON UPDATE CASCADE
                ON DELETE CASCADE,
            FOREIGN KEY(client_ID)        REFERENCES Client(ID)
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn clients(&self) -> Option<&[Client]> {
        self.clients.as_deref()
    }

    pub async fn query_clients(&self, db: &DB) -> Result<Vec<ClientRef>> {
        db.connection(|conn| {
            let mut stmt = conn.prepare_cached(
                r"
                SELECT Client.ID, Client.adresse FROM Client 
                    JOIN MM_ClientGroupClient ON MM_ClientGroupClient.client_ID = Client.ID
                    WHERE MM_ClientGroupClient.client_group_ID = ?",
            )?;

            let columns = columns_from_statement(&stmt);

            let query = stmt.query_and_then([self.id.clone()], |row| -> Result<ClientRef> {
                let client = from_row_with_columns::<Client>(row, &columns)?;
                Ok(ClientRef::from(client))
            })?;

            let clients = Result::from_iter(query)?;

            Ok(clients)
        })
        .await
    }

    pub async fn fetch_clients(&mut self, db: &DB) -> Result<()> {
        db.connection(|conn| {
            let mut stmt = conn.prepare_cached(
                r"
                SELECT Client.ID, Client.adresse FROM Client 
                    JOIN MM_ClientGroupClient ON MM_ClientGroupClient.client_ID = Client.ID
                    WHERE MM_ClientGroupClient.client_group_ID = ?",
            )?;

            let columns = columns_from_statement(&stmt);

            let clients = Result::from_iter(stmt.query_and_then([self.id.clone()], |row| {
                from_row_with_columns::<Client>(row, &columns)
            })?)?;

            self.clients = Some(clients);

            Ok(())
        })
        .await
    }

    pub async fn create(name: String, db: &DB) -> Result<Self> {
        let this = Self::new(name);

        db.connection(|conn| {
            let mut stmt =
                conn.prepare_cached("INSERT INTO ClientGroup (ID, name) VALUES (:id, :name)")?;

            stmt.execute(to_params_named(&this)?.to_slice().as_slice())?;

            Ok(())
        })
        .await?;

        Ok(this)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    // pub fn get(id: String, db: &DB) -> Result<Option<Self>> {
    //     let mut stmt = db
    //         .connection()
    //         .prepare_cached("SELECT * FROM ClientGroup WHERE ID = ?")?;

    //     let columns = columns_from_statement(&stmt);

    //     let mut rows = stmt
    //         .query_and_then([id], |row| from_row_with_columns::<Self>(row, &columns))
    //         .unwrap();

    //     Ok(rows.next().transpose()?)
    // }

    pub async fn add_client(&mut self, id: String, db: &DB) -> Result<()> {
        db.connection(|conn| {
            let mut stmt = conn.prepare_cached(
                "INSERT INTO MM_ClientGroupClient (client_group_ID, client_ID) VALUES (?, ?)",
            )?;

            stmt.execute((self.id.clone(), id))?;

            Ok(())
        })
        .await
    }

    pub async fn add_clients(&mut self, ids: &[String], db: &DB) -> Result<()> {
        db.connection(|conn| {
            let mut stmt = conn.prepare_cached(
                "INSERT INTO MM_ClientGroupClient (client_group_ID, client_ID) VALUES (?, ?)",
            )?;

            for id in ids {
                stmt.execute((self.id.clone(), id))?;
            }

            Ok(())
        })
        .await
    }

    pub async fn remove_client(&mut self, id: String, db: &DB) -> Result<()> {
        db.connection(|conn| {
            let mut stmt = conn.prepare_cached(
                "DELETE FROM MM_ClientGroupClient WHERE client_group_ID = ? AND client_ID = ?",
            )?;

            stmt.execute((self.id.clone(), id))?;

            Ok(())
        })
        .await
    }

    pub async fn remove_clients(&mut self, ids: &[String], db: &DB) -> Result<()> {
        db.connection(|conn| {
            let mut stmt = conn.prepare_cached(
                "DELETE FROM MM_ClientGroupClient WHERE client_group_ID = ? AND client_ID = ?",
            )?;

            for id in ids {
                stmt.execute((self.id.clone(), id))?;
            }

            Ok(())
        })
        .await
    }
}
