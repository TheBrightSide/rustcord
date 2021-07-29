mod state;

use state::ClientState;

use std::error::Error;

use futures::stream::StreamExt;
use std::sync::mpsc;

use http::header::{HeaderMap, HeaderName};

use twilight_gateway::{Intents, Shard, EventTypeFlags, Event};
use twilight_http::Client;

use tui::Terminal;
use tui::backend::CrosstermBackend;
use tui::widgets::{Block, Borders, List, ListItem};
use tui::layout::{Layout, Constraint, Direction};

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
    
    // build gateway websocket client
    let (shard, mut events) = Shard::builder(get_last_from_split(&token).trim(), Intents::all())
        .event_types(EventTypeFlags::all())
        .http_client(http_client.clone())
        .build();
    
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
    
    let (tx, rx) = mpsc::channel::<Event>();
    let mut state: ClientState = ClientState::new();

    let ui_task = tokio::spawn(async move {
        loop {
            // let state_clone = state.clone();

            // event processing section
            let data = rx.try_recv();
            if !data.is_err() {
                state.process_event(data.unwrap());
            }
        }
    });

    shard.start().await?;

    let gateway_task = tokio::spawn(async move {
        loop {
            while let Some(event) = events.next().await {
                tx.send(event).unwrap();
            }
        }
    });
    
    tokio::join!(
        ui_task,
        gateway_task
    );

    Ok(())
}
