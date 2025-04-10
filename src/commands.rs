use std::collections::HashMap;

use crate::*;
use chrono::{Datelike, NaiveDate};

use poise::{
    CreateReply,
    serenity_prelude::{Colour, CreateEmbed, User},
};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Main Command for Birthday-related actions
#[poise::command(
    slash_command,
    prefix_command,
    subcommands("get", "set", "edit", "delete", "list"),
    subcommand_required
)]
pub async fn birthday(_ctx: Context<'_>) -> anyhow::Result<(), Error> {
    Ok(())
}

/// Gets a specific user's birthday.
#[poise::command(slash_command, prefix_command)]
pub async fn get(
    ctx: Context<'_>,
    #[description = "The user who's birthday to get"] user: User,
) -> anyhow::Result<(), Error> {
    let avatar = user.avatar_url().unwrap_or_default();
    let person = get_birthday(user.id.get()).expect(
        "Failed to get birthday. Check if the user exists and if they have a birthday set.",
    );

    let embed = CreateEmbed::new().title(format!("{}'s Birthday", person.name));
    let embed = embed.description(format!(
        "Birthday is {}",
        DateTime::format(&person.birthday, "%d-%m-%Y")
    ));
    let embed = embed.thumbnail(avatar).color(Colour::TEAL);
    let message = CreateReply {
        embeds: vec![embed],
        ..Default::default()
    };
    ctx.send(message).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "The user who's birthday to get"] user: User,
    #[description = "Name"] name: String,
    #[description = "Birthday"] date: String,
) -> anyhow::Result<(), Error> {
    let avatar = user.avatar_url().unwrap_or_default();
    let date = NaiveDate::parse_from_str(&date, "%d-%m-%Y")?;
    let date = DateTime::from_naive_utc_and_offset(date.into(), Utc);
    let person = Person::new(date, name.clone(), user.id.get());
    add_birthday(person)?;
    let embed = CreateEmbed::new()
        .title(format!("Added new Birthday for {}", name))
        .description(format!(
            "Added {} as {}'s birthday",
            DateTime::format(&date, "%d-%m-%Y"),
            name
        ))
        .thumbnail(avatar)
        .color(Colour::DARK_GREEN);
    let reply = CreateReply {
        embeds: vec![embed],
        ..Default::default()
    };
    ctx.send(reply).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn edit(
    ctx: Context<'_>,
    #[description = "The user who's birthday to get"] user: User,
    #[description = "New Birthday"] new_birthday: String,
) -> anyhow::Result<(), Error> {
    let avatar = user.avatar_url().unwrap_or_default();
    let new_birthday = NaiveDate::parse_from_str(&new_birthday, "%d-%m-%Y")?;
    let new_birthday = DateTime::from_naive_utc_and_offset(new_birthday.into(), Utc);
    let _ = edit_birthday(user.id.get(), new_birthday);
    let embed = CreateEmbed::new()
        .title(format!("Edited Birthday for {}", user.name))
        .description(format!(
            "Edited {} as {}'s birthday",
            DateTime::format(&new_birthday, "%d-%m-%Y"),
            user.name
        ))
        .thumbnail(avatar)
        .color(Colour::ORANGE);
    let reply = CreateReply {
        embeds: vec![embed],
        ..Default::default()
    };
    ctx.send(reply).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "The user who's birthday to get"] user: User,
) -> anyhow::Result<(), Error> {
    let _ = delete_birthday(user.id.get());
    let avatar = user.avatar_url().unwrap_or_default();
    let embed = CreateEmbed::new()
        .title(format!("Deleted Birthday for {}", user.name))
        .description(format!("Deleted {}'s birthday", user.name))
        .thumbnail(avatar)
        .color(Colour::RED);
    let reply = CreateReply {
        embeds: vec![embed],
        ..Default::default()
    };
    ctx.send(reply).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn list(ctx: Context<'_>) -> anyhow::Result<(), Error> {
    let birthdays = get_birthdays()?;
    let mut hashmap = HashMap::new();
    for birthday in birthdays {
        let month = birthday.birthday.month();
        let fmt = format!(
            "{} | {}",
            Mention::from(UserId::new(birthday.user_id)),
            birthday.birthday.format("%d-%m-%Y")
        );
        let mut month_vec = hashmap.entry(month).or_insert(vec![]).to_owned();
        month_vec.push(fmt);
        hashmap.insert(month, month_vec.to_vec());
    }
    let embed = CreateEmbed::new().title("Birthdays");
    let mut fields = vec![];
    for (month, fmt) in hashmap {
        let field = (month, fmt.join("\n"), false);
        fields.push(field);
    }
    fields.sort_by(|a, b| a.0.cmp(&b.0));
    let fields = fields.into_iter().map(|(month, fmt, _)| {
        let month = to_month(month);
        (month.name().to_string(), fmt, false)
    });
    let embed = embed
        .fields(fields)
        .color(Colour::GOLD)
        .description("Use `/birthday set` to add your birthday!");
    let reply = CreateReply {
        embeds: vec![embed],
        ..Default::default()
    };
    ctx.send(reply).await?;

    Ok(())
}

