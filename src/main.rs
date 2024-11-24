use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    let email = Message::builder()
        .from("NoBody <matteo.delfour@tsm-tp.fr>".parse().unwrap())
        .to("Test <test@tsm-tp.fr>".parse().unwrap())
        .subject("Happy new year")
        .header(ContentType::TEXT_PLAIN)
        .body(String::from("Be happy!"))
        .unwrap();


    let email2 = Message::builder()
        .from("NoBody <lool@tsm-tp.fr>".parse().unwrap())
        .to("Test <test@tsm-tp.fr>".parse().unwrap())
        .subject("Happy lol year")
        .header(ContentType::TEXT_PLAIN)
        .body(String::from("Be lol!"))
        .unwrap();

    let username = dotenvy::var("SMTP_USERNAME")?;
    let password = dotenvy::var("SMTP_PASSWORD")?;
    let creds = Credentials::new(username, password);

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {e:?}"),
    }

    // Send the email
    match mailer.send(&email2) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {e:?}"),
    }

    Ok(())
}
