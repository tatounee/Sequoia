use color_eyre::eyre::Result;
use tracing::info;
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use sequoia::{
    client::{Client, Group},
    db::DB,
    email::{Email, EmailBuilder},
    mailer::Mailer,
};

fn main() -> Result<()> {
    init()?;

    let db = DB::connect()?;
    db.clean()?;

    // let ids = (0..10)
    //     .map(|i| {
    //         Client::create(&format!("test+{}@tsm-tp.fr", i), &db)
    //             .unwrap()
    //             .id()
    //             .to_owned()
    //     })
    //     .collect::<Vec<String>>();

    // let mut group = Group::create("Golems".into(), &db)?;

    // group.add_clients(&ids[0..5], &db)?;

    // info!("{:?}", group);

    // group.fetch_client(&db)?;
    // info!("{:?}", group);

    let email = EmailBuilder::new()
        .subject("Greeting")
        .plain_body("Hello from <strong>Sequoia</strong>")
        .sender_adresse("Tarak <matteo.delfour@tsm-tp.fr>")?
        .tags(vec!["JE".to_owned(), "MRI".to_owned(), "ff".to_owned()])?
        .create(&db)?;

    for i in 0..10 {
        EmailBuilder::new()
            .subject(&format!("G{i}"))
            .plain_body("")
            .sender_adresse("a@a.a")?
            .create(&db)?;
    }

    let em2 = Email::get_one(email.id_(), &db)?;

    info!("{email:?}");
    info!("{em2:?}");

    // let client = Client::create("test+aa@tsm-tp.fr", &db)?;

    // let mailer = Mailer::new(&db)?;

    // mailer.send(&email, &mut client.into())?;
    // mailer.send(&email, &mut group.into())?;

    // let email = Message::builder()
    //     .from("NoBody <matteo.delfour@tsm-tp.fr>".parse().unwrap())
    //     .to("Test <test@tsm-tp.fr>".parse().unwrap())
    //     .subject("Happy new year")
    //     .header(ContentType::TEXT_PLAIN)
    //     .body(String::from("Be happy!"))
    //     .unwrap();

    // let email2 = Message::builder()
    //     .from("NoBody <lol@tsm-tp.fr>".parse().unwrap())
    //     .to("Test <test@tsm-tp.fr>".parse().unwrap())
    //     .subject("Happy lol year")
    //     .header(ContentType::TEXT_PLAIN)
    //     .body(String::from("Be lol!"))
    //     .unwrap();

    // let username = dotenvy::var("SMTP_USERNAME")?;
    // let password: String = dotenvy::var("SMTP_PASSWORD")?;
    // let creds = Credentials::new(username, password);

    // // Open a remote connection to gmail
    // let mailer = SmtpTransport::relay("smtp.gmail.com")
    //     .unwrap()
    //     .credentials(creds)
    //     .build();

    // // Send the email
    // match mailer.send(&email) {
    //     Ok(_) => println!("Email sent successfully!"),
    //     Err(e) => panic!("Could not send email: {e:?}"),
    // }

    // // Send the email
    // match mailer.send(&email2) {
    //     Ok(_) => println!("Email sent successfully!"),
    //     Err(e) => panic!("Could not send email: {e:?}"),
    // }

    Ok(())
}

fn init() -> Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv()?;

    // let subscriber = FmtSubscriber::builder()
    //     .with_max_level(Level::DEBUG)
    //     .with(EnvFilter::from_env("sequoia"))
    //     .finish();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(ErrorLayer::default())
        .with(EnvFilter::from_default_env())
        .init();

    // tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    Ok(())
}
