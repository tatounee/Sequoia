use std::sync::OnceLock;

use color_eyre::eyre::Result;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::client::{Client, Group};
use crate::email::Email;

pub struct Mailer {
    smtp_transport: SmtpTransport,
}

static mut MAILER_SINGLETON: OnceLock<Mailer> = OnceLock::new();

impl Mailer {
    pub fn new() -> Result<Self> {
        let username = dotenvy::var("SMTP_USERNAME")?;
        let password = dotenvy::var("SMTP_PASSWORD")?;
        let creds = Credentials::new(username, password);

        // Open a remote connection to gmail
        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(creds)
            .build();

        mailer.test_connection()?;

        Ok(Self {
            smtp_transport: mailer,
        })
    }

    pub fn send(
        &self,
        email: &Email,
        receiver: &Receiver,
    ) -> Result<()> {
        match receiver {
            Receiver::Client(client) => self.send_to_client(email, client),
            Receiver::Group(group) => self.send_to_group(email, group),
        }
    }

    fn send_to_client(
        &self,
        email: &Email,
        client: &Client,
    ) -> Result<()> {
        // Unwrap is safe because from and to are provided
        let email = Message::builder()
            .from(email.sender_adresse().parse().unwrap())
            .to(client.adresse().parse().unwrap())
            .subject(email.subject())
            .header(ContentType::TEXT_HTML)
            .body(email.body())
            .unwrap();

        // debug!(self.smtp_transport.);
        self.smtp_transport.send(&email)?;

        Ok(())
    }

    fn send_to_group(
        &self,
        email: &Email,
        group: &Group,
    ) -> Result<()> {
        // TODO: Gérer les erreurs pour les cas où tous les mails ne sont pas envoyé.
        // TODO: Gérer le cas où les clients du group n'ont pas été fetch.

        group
            .clients()
            .unwrap()
            .iter()
            .try_for_each(|client| self.send_to_client(email, client))
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

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     dotenvy::dotenv()?;

//     let email = Message::builder()
//         .from("NoBody <matteo.delfour@tsm-tp.fr>".parse().unwrap())
//         .to("Test <test@tsm-tp.fr>".parse().unwrap())
//         .subject("Happy new year")
//         .header(ContentType::TEXT_PLAIN)
//         .body(String::from("Be happy!"))
//         .unwrap();

//     let email2 = Message::builder()
//         .from("NoBody <lol@tsm-tp.fr>".parse().unwrap())
//         .to("Test <test@tsm-tp.fr>".parse().unwrap())
//         .subject("Happy lol year")
//         .header(ContentType::TEXT_PLAIN)
//         .body(String::from("Be lol!"))
//         .unwrap();

//     let username = dotenvy::var("SMTP_USERNAME")?;
//     let password = dotenvy::var("SMTP_PASSWORD")?;
//     let creds = Credentials::new(username, password);

//     // Open a remote connection to gmail
//     let mailer = SmtpTransport::relay("smtp.gmail.com")
//         .unwrap()
//         .credentials(creds)
//         .build();

//     // Send the email
//     match mailer.send(&email) {
//         Ok(_) => println!("Email sent successfully!"),
//         Err(e) => panic!("Could not send email: {e:?}"),
//     }

//     // Send the email
//     match mailer.send(&email2) {
//         Ok(_) => println!("Email sent successfully!"),
//         Err(e) => panic!("Could not send email: {e:?}"),
//     }

//     Ok(())
// }
