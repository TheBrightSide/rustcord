use std::error::Error;
use std::sync::atomic::*;
use std::sync::Arc;
use futures::stream::StreamExt;
use http::header::{HeaderMap, HeaderName};
use cursive::views::{Dialog, TextView};
use twilight_gateway::{Intents, Shard, EventTypeFlags, Event, cluster::{Cluster, ShardScheme}};
use twilight_http::Client;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};

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

fn get_last_from_split(inp: &str) -> String {
    let mut split = inp.rsplit(" ");

    split.next().unwrap().to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut token = String::new();

    println!("token login (email and password login not supported yet :/)");
    std::io::stdin().read_line(&mut token).expect("lol how'd this error");

    token = token.trim().into();

    // initialize headers prepended with every http request
    let mut default_headers = HeaderMap::new();

    // default_headers.insert(
    //     HeaderName::from_lowercase(b"user-agent").unwrap(),
    //     fetch_user_agent().trim().parse().unwrap()
    // );
    default_headers.insert(
        HeaderName::from_lowercase(b"authorization").unwrap(),
        token.parse().unwrap()
    );
    
    // build http client
    let http_client = Client::builder()
        .default_headers(default_headers)
        .build();

    println!("{:?}", http_client.current_user().await);
    
    // build gateway websocket client
    let (shard, mut events) = Shard::builder(get_last_from_split(&token).trim(), Intents::all())
        .event_types(EventTypeFlags::all())
        .http_client(http_client.clone())
        .build();

    shard.start().await?;

    // define shard scheme
    // let scheme = ShardScheme::Auto;

    // build cluster
    // let (cluster, mut events) = Cluster::builder(get_last_from_split(&token), Intents::all())
    //     .http_client(http_client)
    //     .shard_scheme(scheme)
    //     .build()
    //     .await?;

    // let cluster_spawn = cluster.clone();

    // tokio::spawn(async move {
    //     cluster_spawn.up().await;
    // });

    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::all())
        .build();
    
    let mut last_msg_id = Arc::new(AtomicU64::new(0));
    
    tokio::spawn(async move {
        // let mut siv = cursive::default();
            
        // siv.add_layer(
        //     Dialog::around(TextView::new(format!("{:?}", last_msg_id)))
        //         .title("title")
        //         .button("quit", |s| s.quit())
        // );

        // siv.run();
    });


    loop {
        while let Some(event) = events.next().await {
            cache.update(&event);

            match event {
                Event::MessageCreate(msg) => {
                    // println!("msg with id {} has been created", msg.id);
                    last_msg_id.store(msg.id.to_string().parse().unwrap(), Ordering::SeqCst);
                },
                _ => {}
            }
        }
    } 
}
