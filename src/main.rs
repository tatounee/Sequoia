#![feature(async_closure)]

use std::sync::Arc;

use color_eyre::eyre::Result;
use tracing::{info, instrument};
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use sequoia::{
    db::DB,
    email::EmailBuilder,
    mailer::Mailer,
    scheduler::{
        trigger::{Counter, CounterTrigger, DatetimeTrigger, NaiveTime, PartialDate, Trigger},
        Scheduler,
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    init()?;

    let db = Box::new(DB::connect().await?);
    let db: &'static DB = Box::leak(db);
    db.clean().await?;

    let mailer = Mailer::new(db)?;

    let mut scheduler = Scheduler::new(mailer);

    let _email = std::sync::Arc::new(
        EmailBuilder::new()
            .subject("Greeting")
            .plain_body("Hello from <strong>Sequoia</strong>")
            .sender_adresse("Tarak <matteo.delfour@tsm-tp.fr>")?
            .tags(vec!["JE".to_owned(), "MRI".to_owned(), "ff".to_owned()])?
            .create(db)
            .await?,
    );

    // let r = Client::create("test+aa@tsm-tp.fr", db).await?.into();

    let mut trigger = CounterTrigger::new(Counter::Finit(10), || {
        Box::new(DatetimeTrigger::new(
            PartialDate::new_y(2024),
            NaiveTime::from_hms_opt(16, 1, 40).unwrap(),
        ))
    });

    // let mut trigger = DatetimeTrigger::new(
    //     PartialDate::new_y(2024),
    //     NaiveTime::from_hms_opt(15, 41, 40).unwrap(),
    // );

    info!("Generation = {}", trigger.generation());
    trigger.forward_generation(10);
    info!("Generation = {}", trigger.generation());

    #[instrument(skip(_mailer))]
    async fn send_email(generation: u64, _mailer: Arc<Mailer<'static>>) {
        info!("Receive {generation}");
    }

    scheduler.register_trigger_with_action(Box::new(trigger), send_email);

    for i in 0.. {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        println!("tick {}", i);
        if i % 10 == 0 {
            // info!("Generation = {}", trigger.generation());
        }
    }

    Ok(())

    // let send_email = |generation: u64, mailer: &Mailer<'_>| async move {
    //     info!("Receive {generation}");
    // };
    // scheduler.register_trigger_with_action(Box::new(trigger), Box::new(send_email));

    // let now = tokio::time::Instant::now();
    // debug!("now = {now:?}");

    // tokio::time::pause();
    // tokio::time::advance(Duration::from_secs(3600 * 24)).await;
    // tokio::time::resume();

    // loop {
    //     let now = tokio::time::Instant::now();
    //     debug!("now = {now:?}");

    //     tokio::time::pause();
    //     tokio::time::advance(Duration::from_secs(3600 * 24)).await;
    //     tokio::time::resume();

    //     tokio::time::sleep(Duration::from_secs(1)).await;
    // }

    // Ok(())

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
