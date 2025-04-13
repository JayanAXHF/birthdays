mod commands;
use std::sync::Arc;

use birthdays::*;
use chrono::{DateTime, Datelike, Utc};
use poise::serenity_prelude::{
    self as serenity, ChannelId, CreateEmbedFooter, CreateMessage, Mention, Mentionable, UserId,
};
use rusqlite::Connection;
use tracing::{error, info};

#[derive(Debug)]
struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

fn init_db() -> anyhow::Result<()> {
    let conn = Connection::open("./birthdays.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS birthdays (
            user_id INTEGER PRIMARY KEY,
            birthday INTEGER NOT NULL,
            name TEXT NOT NULL
        )",
        (),
    )?;
    Ok(())
}

fn add_birthday(person: Person) -> anyhow::Result<()> {
    let user_id = person.user_id;
    let birthday = person.get_date_int();
    let name = person.name;
    let conn = Connection::open("./birthdays.db")?;
    conn.execute(
        "INSERT INTO birthdays (user_id, birthday, name) VALUES (?1, ?2, ?3)",
        (user_id, birthday, name),
    )?;
    Ok(())
}

fn get_birthday(user_id: u64) -> anyhow::Result<Person> {
    let conn = Connection::open("./birthdays.db")?;
    let mut stmt = conn.prepare("SELECT * FROM birthdays WHERE user_id = ?")?;
    let mut rows = stmt.query([user_id])?;
    let person = rows.next()?;
    if person.is_none() {
        return Err(anyhow::anyhow!("No birthday found for user <@{}>", user_id));
    }
    Ok(Person::new(
        DateTime::from_timestamp(person.unwrap().get(1)?, 0).unwrap(),
        person.unwrap().get(2)?,
        user_id,
    ))
}

fn edit_birthday(user_id: u64, birthday: DateTime<Utc>) -> anyhow::Result<()> {
    let birthday = birthday.timestamp();

    let conn = Connection::open("./birthdays.db")?;
    let name: String = conn.query_row(
        "SELECT name FROM birthdays WHERE user_id = ?1",
        [user_id],
        |row| row.get(0),
    )?;
    conn.execute(
        "UPDATE birthdays SET birthday = ?1, name = ?2 WHERE user_id = ?3",
        (birthday, name, user_id),
    )?;
    Ok(())
}

fn delete_birthday(user_id: u64) -> Result<(), Error> {
    let conn = Connection::open("./birthdays.db")?;
    conn.execute("DELETE FROM birthdays WHERE user_id = ?1", (user_id,))?;
    Ok(())
}

fn get_birthdays() -> Result<Vec<Person>, Error> {
    let conn = Connection::open("./birthdays.db")?;
    let mut stmt = conn.prepare("SELECT * FROM birthdays")?;
    let mut birthdays = vec![];
    for row in stmt.query_map([], Person::from_row)? {
        birthdays.push(row?);
    }
    Ok(birthdays)
}

#[tokio::main]
#[tracing::instrument]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();
    tracing_subscriber::fmt::init();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![crate::commands::birthday(), crate::commands::age()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let ctx_2 = Arc::new(ctx.clone());
                tokio::task::spawn(async move {
                    if let Err(e) = init_db() {
                        error!("Failed to initialize database: {}", e);
                    }
                    let mut interval =
                        tokio::time::interval(std::time::Duration::from_secs(86_400));
                    loop {
                        let announcements_channel_id = ChannelId::new(1338908758218117210);
                        info!("Starting birthdays...");
                        interval.tick().await;
                        let birthdays_today = fetch_birthdays_today().unwrap();
                        info!(?birthdays_today);

                        if !birthdays_today.is_empty() {
                            for birthday in birthdays_today {
                                let embed = build_embed(birthday);
                                let message =
                                    CreateMessage::new().embed(embed).content("@everyone");
                                let ctx = Arc::clone(&ctx_2);
                                {
                                    send(
                                        message,
                                        announcements_channel_id,
                                        Arc::unwrap_or_clone(ctx),
                                    )
                                    .await
                                    .unwrap();
                                }
                            }
                        }
                    }
                });
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

fn build_embed(person: Person) -> serenity::builder::CreateEmbed {
    serenity::builder::CreateEmbed::default()
        .title(format!("{}'s Birthday", person.name))
        .description(format!(
            "@everyone, It's {}'s birthday! Don't forget the chocolates and the birthday bumps!",
            Mention::from(UserId::new(person.user_id))
        ))
        .color(serenity::Colour::GOLD)
        .footer(CreateEmbedFooter::new(format!("{}", person.birthday)))
}

async fn send(
    message: CreateMessage,
    channel_id: ChannelId,
    ctx: serenity::Context,
) -> anyhow::Result<()> {
    channel_id.send_message(ctx, message).await?;
    Ok(())
}

fn fetch_birthdays_today() -> anyhow::Result<Vec<Person>> {
    let conn = rusqlite::Connection::open("./birthdays.db").unwrap();
    let mut stmt = conn.prepare("SELECT * FROM birthdays").unwrap();
    let mut rows = stmt.query(()).unwrap();
    let date = Utc::now();

    let mut birthdays_today = vec![];
    while let Some(person) = rows.next().unwrap() {
        let user_id: i64 = person.get(0).unwrap();
        let birthday: i64 = person.get(1).unwrap();
        let name: String = person.get(2).unwrap();

        let birthday = DateTime::from_timestamp(birthday, 0).unwrap();

        if birthday.month() == date.month() && date.day() == birthday.day() {
            birthdays_today.push(Person::new(birthday, name, user_id.try_into()?));
        }
    }

    Ok(birthdays_today)
}
