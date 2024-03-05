use std::path::Path;
use std::{env, time};

use serenity::all::{
    CreateAttachment, CreateEmbed, CreateEmbedFooter, CreateMessage, EditMessage, Timestamp,
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

fn writeOut(content: &str) {
    let data = format!(
        "
        #import \"@preview/physica:0.9.2\" : *
        #set page(width: auto, height: auto, margin: .1em, fill: rgb(49, 51,56 ,255))
        #set text(size: 24pt, fill: white)

        $ {content} $  
    "
    );

    fs::write("./test.typ", data).expect("Unable to write file");

    let mut typst = Command::new("typst");
    typst.arg("compile").arg("test.typ");
    typst.arg("./out.png");
    typst.output().expect("failed to execute process");
}

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("?math") {
            // Sending a message can fail, due to a network error, an authentication error, or lack
            // of permissions to post in the channel, so log to stdout when some error happens,
            // with a description of it.
            let body: Vec<&str> = msg.content.split("?math ").collect();
            if (body.len() == 2) {
                println!("Math message sent: {}", body[1]);

                writeOut(body[1]);
                // let ten_millis = time::Duration::from_millis(150);
                // thread::sleep(ten_millis);
                //let f = [(&tokio::fs::File::open("./out.png").await, "out.png")];

                let builder = CreateMessage::new()
                    .add_file(CreateAttachment::path("./out.png").await.unwrap());
                let mut msg = msg.channel_id.send_message(&ctx.http, builder).await;
                match msg {
                    Ok(mut msg) => {
                        let ten_millis = time::Duration::from_millis(2000);
                        thread::sleep(ten_millis);
                        writeOut("c");
                        let builder2 = EditMessage::new()
                            .remove_all_attachments()
                            .new_attachment(CreateAttachment::path("./out.png").await.unwrap());
                        msg.edit(ctx, builder2).await;
                    }
                    Err(msg) => {}
                }

                //return
                //writeOut("");
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // thread::spawn(move || {
    //     let mut typst = Command::new("typst");
    //     typst.arg("watch").arg("test.typ");
    //     typst.arg("./out.png");
    //     typst.output().expect("failed to execute process");
    // });

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}

//
