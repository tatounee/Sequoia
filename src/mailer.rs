use std::time::{SystemTime, UNIX_EPOCH};

use color_eyre::eyre::Result;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::PoolConfig;
use lettre::{Message, SmtpTransport, Transport};
use tracing::debug;

use crate::client::{Client, Group};
use crate::db::DB;
use crate::email::Email;

pub struct Mailer<'a> {
    smtp_transport: SmtpTransport,
    db: &'a DB,
}

impl<'a> Mailer<'a> {
    pub const CREATE_TABLES: &'static str = r"
        CREATE TABLE IF NOT EXISTS MM_EmailClient (
            email_ID   TEXT,
            client_ID  TEXT,
            timestamp  INTEGER,
            FOREIGN KEY(email_ID)  REFERENCES Email(ID)
                ON UPDATE CASCADE
                ON DELETE CASCADE,
            FOREIGN KEY(client_ID)  REFERENCES Client(ID)
                ON UPDATE CASCADE
                ON DELETE CASCADE
        ) STRICT;

        CREATE TABLE IF NOT EXISTS MM_EmailClientGroup (
            email_ID         TEXT,
            client_group_ID  TEXT,
            timestamp        INTEGER,
            FOREIGN KEY(email_ID)  REFERENCES Email(ID)
                ON UPDATE CASCADE
                ON DELETE CASCADE,
            FOREIGN KEY(client_group_ID)  REFERENCES ClientGroup(ID)
                ON UPDATE CASCADE
                ON DELETE CASCADE
        ) STRICT;
    ";

    pub fn new(db: &'a DB) -> Result<Self> {
        let username = dotenvy::var("SMTP_USERNAME")?;
        let password = dotenvy::var("SMTP_PASSWORD")?;
        let creds = Credentials::new(username, password);

        // Open a remote connection to gmail
        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(creds)
            .pool_config(PoolConfig::new())
            .build();

        mailer.test_connection()?;

        Ok(Self {
            smtp_transport: mailer,
            db,
        })
    }

    pub async fn send(&self, email: &Email, receiver: &mut Receiver) -> Result<()> {
        match receiver {
            Receiver::Client(client) => self.send_to_client(email, client)?,
            Receiver::Group(group) => self.send_to_group(email, group, self.db).await?,
        }

        self.write_sending(email, receiver).await?;

        Ok(())
    }

    fn send_to_client(&self, email: &Email, client: &Client) -> Result<()> {
        debug!(
            "Send email to client. email = {}, client = {}",
            email.id(),
            client.id()
        );

        // Unwrap is safe because from and to are provided
        let message = Message::builder()
            .from(email.sender_adresse().parse().unwrap())
            .to(client.adresse().parse().unwrap())
            .subject(email.subject())
            .header(ContentType::TEXT_HTML)
            .body(email.body())
            .unwrap();

        // debug!(self.smtp_transport.);
        self.smtp_transport.send(&message)?;

        Ok(())
    }

    async fn send_to_group(&self, email: &Email, group: &Group, db: &DB) -> Result<()> {
        // TODO: Gérer les erreurs pour les cas où tous les mails ne sont pas envoyé.
        // TODO: Gérer le cas où les clients du group n'ont pas été fetch.

        debug!(
            "Send email to group. email = {}, group = {}",
            email.id(),
            group.id()
        );

        group.query_clients(db).await?;

        group
            .clients()
            .unwrap()
            .iter()
            .try_for_each(|client| self.send_to_client(email, client))
    }

    async fn write_sending(&self, email: &Email, receiver: &Receiver) -> Result<()> {
        match receiver {
            Receiver::Client(client) => {
                debug!(
                    "Write to database email sent to client. email={}, client={}",
                    email.id(),
                    client.id()
                );

                self.db.connection(|conn| {
                    let mut stmt = conn.prepare_cached(
                        r"
                        INSERT INTO MM_EmailClient (email_ID, client_ID, timestamp) VALUES (?, ?, ?)
                    ",
                    )?;
    
                    let params = (
                        email.id(),
                        client.id(),
                        // Unwrap in safe because `UNIX_EPOCH` is 0 and thus less than `SystemTime::now()`
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    );
    
                    stmt.execute(params)?;

                    Ok(())
                }).await?;
            }
            Receiver::Group(group) => {
                debug!(
                    "Write to database email sent to group. email={}, group={}",
                    email.id(),
                    group.id()
                );

                self.db.connection(|conn| {
                    let mut stmt = conn.prepare_cached(
                        r"
                        INSERT INTO MM_EmailClientGroup (email_ID, client_group_ID, timestamp) VALUES (?, ?, ?)
                    ",
                    )?;
    
                    let params = (
                        email.id(),
                        group.id(),
                        // Unwrap in safe because `UNIX_EPOCH` is 0 and thus less than `SystemTime::now()`
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    );
    
                    stmt.execute(params)?;

                    Ok(())
                }).await?;

            }
        }

        Ok(())
    }
}

pub enum Receiver {
    Client(Client),
    Group(Group),
}

impl From<Client> for Receiver {
    fn from(value: Client) -> Self {
        Self::Client(value)
    }
}

impl From<Group> for Receiver {
    fn from(value: Group) -> Self {
        Self::Group(value)
    }
}
