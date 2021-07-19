use twilight_gateway::{Intents, Shard, EventTypeFlags, Event};
use twilight_http::Client;
use http::header::{HeaderMap, HeaderName};
use cursive::views::{Dialog, TextView};

fn fetch_user_agent() -> String {
    if cfg!(unix) {
        if std::env::consts::OS == "macos" {
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_4) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.164 Safari/537.36".into()
        } else {
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.164 Safari/537.36".into()
        }
    } else if cfg!(windows) {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.164 Safari/537.36".into()
    } else {
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.164 Safari/537.36".into()
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    // const ALL_INTENTS = GUILDS | GUILD_MEMBERS | GUILD_BANS | GUILD_EMOJIS | GUILD_INTEGRATIONS | GUILD_WEBHOOKS | GUILD_INVITES | GUILD_VOICE_STATES | GUILD_PRESENCES | GUILD_MESSAGES | GUILD_MESSAGE_REACTIONS | GUILD_MESSAGE_TYPING | DIRECT_MESSAGES | DIRECT_MESSAGE_REACTIONS | DIRECT_MESSAGE_TYPING
    let mut token = String::new();

    println!("token login (email and password login not supported yet :/)");
    std::io::stdin().read_line(&mut token).expect("lol how'd this error");

    let mut default_headers = HeaderMap::new();

    default_headers.insert(
        HeaderName::from_lowercase(b"user-agent").unwrap(),
        fetch_user_agent().trim().parse().unwrap()
    );

    default_headers.insert(
        HeaderName::from_lowercase(b"authorization").unwrap(),
        token.trim().parse().unwrap()
    );
    
    let http_client = Client::builder()
        .default_headers(default_headers)
        .build();
    
    let (shard, mut events) = Shard::builder(&token, Intents::empty())
        // .event_types(event_types)
        .http_client(http_client)
        .build();

    shard.start().await.unwrap();

    // while let Some(event) = events.await {
    //     match event {
    //         Event::MessageCreate(message) => {
    //             println!("message received with content: {}", message.content);
    //         },
    //         Event::MessageDelete(message) => {
    //             println!("message with ID {} deleted", message.id);
    //         },
    //         _ => {},
    //     }
    // }

    // let tui_thread = thread::spawn(move || {
    //     let mut siv = cursive::default();

    //     siv.add_layer(Dialog::around(TextView::new("dialog..?"))
    //         .title("title")
    //         .button("quit", |s| s.quit())
    //     );

    //     siv.run();
    // });

    // tui_thread.join().unwrap();
}