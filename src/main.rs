use std::path::Path;
use std::{env, time};

use serenity::all::{
    ChannelId, CreateAttachment, CreateEmbed, CreateEmbedFooter, CreateMessage, EditMessage,
    MessageUpdateEvent, Timestamp,
};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::fs;
use std::io::{self, Write};
use std::process::Command;
use std::thread;

struct Handler;

/*
Write {content} out to test.typ and compile to png. Returns an option, None if writing was successful, and the
String with the compilation error if unsuccessful
*/
fn writeOut(content: &str) -> Option<String> {
    let data = format!(
        "
#import \"@preview/physica:0.9.2\" : *
#set page(width: auto, height: auto, margin: .1em, fill: rgb(49, 51,56 ,255))
#set text(size: 24pt, fill: white)

$ {content} $  
    "
    );
    fs::write("./test.typ", data).expect("Unable to write file");
    let mut typst = Command::new("./typst/typst");
    typst.arg("compile").arg("./test.typ");
    typst.arg("./out.png");
    let a = typst.output().expect("failed to execute process");

    if a.status.success() {
        return None;
    } else {
        let result_string = std::str::from_utf8(&a.stderr)
            .ok() // Convert the Result to an Option, disregarding the error
            .map(|s| s.to_string())?; // Convert the str to String if it's Some
        return Some(result_string);
    }
}

/*
Send {body} to {chid} (channel id)
*/
async fn sendMessage(ctx: Context, body: &str, chid: ChannelId) {
    let a = writeOut(body);
    let builder: CreateMessage;
    match a {
        None => {
            builder =
                CreateMessage::new().add_file(CreateAttachment::path("./out.png").await.unwrap());
        }
        Some(s) => {
            builder = CreateMessage::new().content(format!("```{s}```"));
        }
    }
    let mut msg = chid.send_message(&ctx.http, builder).await;
    writeOut("");
}

#[async_trait]
impl EventHandler for Handler {
    /*
    Send new message upon a message being edited
    TODO: Keep track of already sent messages and edit the corresponding one rather than sending a whole new one
     */
    async fn message_update(
        &self,
        ctx: Context,
        old_if_available: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        match event.content {
            Some(a) => {
                let body: Vec<&str> = a.split("?math ").collect();
                println!("Math message update: {}", a);
                sendMessage(ctx, body[1], event.channel_id).await;
            }
            none => {}
        }
    }

    /*
    Send message with typst png if message starts with ?math
     */
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("?math") {
            let body: Vec<&str> = msg.content.split("?math ").collect();
            if (body.len() == 2) {
                println!("Math message sent: {}", body[1]);
                sendMessage(ctx, body[1], msg.channel_id).await;
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // other approach, have thread watch for faster compilation
    // has issue of not knowing when compilation is finished

    // thread::spawn(move || {
    //     let mut typst = Command::new("typst");
    //     typst.arg("watch").arg("test.typ");
    //     typst.arg("./out.png");
    //     typst.output().expect("failed to execute process");
    // });

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
