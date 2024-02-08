//! Holds event hooks for the bot.

use std::collections::HashSet;
use serenity::all::{Message, StandardFramework, UserId};
use serenity::framework::standard::macros::hook;
use serenity::framework::standard::{CommandError, Configuration};
use serenity::prelude::*;
use crate::bot::groups;
use crate::bot::structures::Config;

#[hook]
async fn after(
    ctx: &Context,
    message: &Message,
    cmd_name: &str,
    error: Result<(), CommandError>
) {

}

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!("Got command '{}' by user '{}'", command_name, msg.author.name);

    true // if `before` returns false, command processing doesn't happen.
}


/// Sets up the given framework for the bot.
pub fn setup_framework(config: Config, owners: HashSet<UserId>) -> StandardFramework {
    let mut framework = StandardFramework::new()
        .after(after)
        .before(before)
        .group(&groups::general::GENERAL_GROUP);
    framework.configure(
        Configuration::new()
            .with_whitespace(true)
            .prefixes(config.prefixes)
            .owners(owners)
    );
    framework
}
