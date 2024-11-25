use color_eyre::eyre::Result;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

fn main() -> Result<()> {
    init()?;

    // let db = DB::connect("sequoia.db")?;
    // db.clean()?;

    // let ids = (0..10).map(|i| {
    //     Client::create(&format!("test+{}@tsm-tp.fr", i), &db).unwrap().id_()
    // }).collect::<Vec<String>>();

    // let mut group = Group::create("Golems".into(), &db)?;

    // group.add_clients(&ids[0..5], &db)?;

    // info!("{:?}", group);

    // group.fetch_client(&db)?;
    // info!("{:?}", group);

    // let email = EmailBuilder::new()
    //     .subject("Greeting")
    //     .plain_body("Hello from <strong>Sequoia</strong>")
    //     .sender_adresse("Tarak <matteo.delfour@tsm-tp.fr>")?
    //     .build();
    // let client = Client::new("test@tsm-tp.fr")?;

    // let mailer = Mailer::new()?;

    // mailer.send(&email, &client.into())?;

    // let email = Message::builder()
    // .from("NoBody <matteo.delfour@tsm-tp.fr>".parse().unwrap())
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

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    Ok(())
}
