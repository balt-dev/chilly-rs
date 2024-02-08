use std::time::Instant;
use serenity::all::CurrentUser;
use serenity::prelude::TypeMapKey;

pub struct StartedTime;

impl TypeMapKey for StartedTime {
    type Value = Instant;
}

pub struct BotUser;

impl TypeMapKey for BotUser {
    type Value = CurrentUser;
}
