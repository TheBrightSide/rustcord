mod state;

use state::ClientState;

use std::error::Error;
use std::io;

use futures::stream::StreamExt;
use std::sync::mpsc;

use http::header::{HeaderMap, HeaderName};

use twilight_gateway::{Event, EventTypeFlags, Intents, Shard};
use twilight_http::Client;

use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, ListItem};
use tui::Terminal;

#[allow(dead_code)]
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
    let mut split = inp.rsplit(' ');

    split.next().unwrap().to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut token = String::new();

    println!("token login (email and password login not supported yet :/)");
    std::io::stdin()
        .read_line(&mut token)
        .expect("lol how'd this error");

    token = token.trim().into();

    // initialize headers prepended with every http request
    let mut default_headers = HeaderMap::new();

    // default_headers.insert(
    //     HeaderName::from_lowercase(b"user-agent").unwrap(),
    //     fetch_user_agent().trim().parse().unwrap()
    // );
    default_headers.insert(
        HeaderName::from_lowercase(b"authorization").unwrap(),
        token.parse().unwrap(),
    );

    // build http client
    let http_client = Client::builder().default_headers(default_headers).build();

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
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.clear().unwrap();

        loop {
            terminal
                .draw(|frame| {
                    if state.clone().authenticated {
                        let general_layout = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                            .split(frame.size());

                        let sidebar_layout = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                            .split(general_layout[0]);

                        let guilds = state.clone().get_guilds();
                        let guild_list_items: Vec<ListItem> = guilds
                            .clone()
                            .into_iter()
                            .map(|guild| ListItem::new(guild.name))
                            .rev()
                            .collect();

                        let guild_list = List::new(guild_list_items)
                            .block(Block::default().title("Guilds").borders(Borders::ALL))
                            .style(Style::default().fg(Color::White))
                            .highlight_style(Style::default().add_modifier(Modifier::ITALIC));

                        let mut channel_list_items: Vec<ListItem> = vec![];

                        if !guilds.is_empty() {
                            let guild_channels =
                                state.clone().get_channels(guilds[0].clone().id).unwrap();

                            channel_list_items = guild_channels
                                .clone()
                                .into_iter()
                                .map(|channel| ListItem::new(channel.name().to_string()))
                                .rev()
                                .collect();
                        }

                        let channel_list = List::new(channel_list_items)
                            .block(Block::default().title("Channels").borders(Borders::ALL))
                            .style(Style::default().fg(Color::White))
                            .highlight_style(Style::default().add_modifier(Modifier::ITALIC));

                        let message_block =
                            Block::default().title("Messages").borders(Borders::ALL);

                        frame.render_widget(guild_list, sidebar_layout[0]);
                        frame.render_widget(channel_list, sidebar_layout[1]);
                        frame.render_widget(message_block, general_layout[1]);
                    } else {
                        let block_size = frame.size();

                        let block = Block::default()
                            .title("logging in...")
                            .borders(Borders::ALL);

                        frame.render_widget(block, block_size);
                    }
                })
                .unwrap();

            // event processing section
            if let Ok(data) = rx.try_recv() {
                state.process_event(data)
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
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

    tokio::join!(ui_task, gateway_task);

    Ok(())
}
