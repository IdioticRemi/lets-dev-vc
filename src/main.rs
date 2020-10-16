use std::env;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use serenity::client::ClientBuilder;
use serenity::model::channel::{ChannelType, PermissionOverwrite, PermissionOverwriteType};
use serenity::model::gateway::Activity;
use serenity::model::id::{ChannelId, GuildId, RoleId};
use serenity::model::Permissions;
use serenity::model::user::OnlineStatus;
use serenity::model::voice::VoiceState;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Une erreur est survenue lors de l'envoi: {:?}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        ctx.set_presence(Option::from(Activity::listening("la plèbe ❤")), OnlineStatus::Idle).await;
    }

    async fn voice_state_update(&self, ctx: Context, guild_id: Option<GuildId>, _: Option<VoiceState>, state: VoiceState) {
        let target_id = ChannelId(env::var("DISCORD_CHANNEL")
            .expect("Expected a DISCORD_CHANNEL in the environment").parse().unwrap());

        let member_role_id = RoleId(env::var("DISCORD_MEMBER_ROLE")
            .expect("Expected a DISCORD_MEMBER_ROLE in the environment").parse().unwrap());

        let chan = ctx.cache.guild_channel(target_id)
            .await.unwrap();

        if state.channel_id == Option::from(chan.id) {
            let user = ctx.cache.user(state.user_id)
                .await.unwrap();

            let guild = ctx.cache.guild(guild_id.unwrap())
                .await.unwrap();

            let member = ctx.cache.member(guild.id, user.id)
                .await.unwrap();

            let member_role = ctx.cache.role(guild.id, member_role_id)
                .await.unwrap();

            let p_chan = guild.create_channel(&ctx.http, |c| c.name(format!("⏱ {}", user.name)).kind(ChannelType::Voice).category(chan.category_id.unwrap()))
                .await.unwrap();

            let _ = p_chan.create_permission(&ctx.http, &PermissionOverwrite {
                allow: Permissions::CONNECT,
                deny: Permissions::default(),
                kind: PermissionOverwriteType::Member(user.id),
            }).await;

            let _ = p_chan.create_permission(&ctx.http, &PermissionOverwrite {
                allow: Permissions::READ_MESSAGES,
                deny: Permissions::CONNECT,
                kind: PermissionOverwriteType::Role(member_role.id),
            }).await;


            let _ = member.move_to_voice_channel(&ctx.http, p_chan)
                .await;
        }
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected DISCORD_TOKEN in the environment");

    let mut client = ClientBuilder::new(&token)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}