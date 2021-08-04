use twilight_model::id::GuildId;
use twilight_model::user::CurrentUser;
use twilight_model::guild::Guild;
use twilight_model::channel::Channel;
use twilight_model::channel::Message;
use twilight_gateway::Event;

#[derive(Clone)]
pub struct StatusGuild {
    available: bool,
    id: GuildId,
    guild: Option<Guild>
}

#[derive(Clone)]
pub struct ClientState {
    pub authenticated: bool,
    pub guilds: Vec<StatusGuild>,
    pub user: Option<CurrentUser>,
    pub current_guild: Option<Guild>,
    pub current_channel: Option<Channel>,
    pub current_channel_messages: Vec<Message>
}

impl ClientState {
    pub const fn new() -> Self {
        ClientState {
            authenticated: false,
            guilds: vec![],
            user: None,
            current_guild: None,
            current_channel: None,
            current_channel_messages: vec![]
        }
    }

    pub fn process_event(&mut self, event: Event) {
        match event {
            Event::Ready(data) => {
                self.authenticated = true;
                self.user = Option::from(data.user);
                self.guilds = data.guilds.into_iter().map(|guild|
                    StatusGuild {
                        available: false,
                        id: guild.id,
                        guild: None
                    }
                ).rev().collect();
            },
            Event::GuildCreate(data) => {
                if self.guild_exists(data.0.id) {
                    self.guilds = self.guilds.clone().into_iter().map(|guild| {
                        if data.clone().0.id == guild.id {
                            StatusGuild {
                                available: true,
                                id: guild.id,
                                guild: Option::from(data.clone().0)
                            }
                        } else {
                            StatusGuild {
                                available: false,
                                id: guild.id,
                                guild: None
                            }
                        }
                    }).rev().collect();
                } else {
                    self.guilds.push(StatusGuild {
                        available: true,
                        id: data.0.id,
                        guild: Option::from(data.0)
                    })
                }
            }
            _ => {}
        }
    }

    pub fn guild_exists(&mut self, id: GuildId) -> bool {
        self.guilds
            .clone()
            .into_iter()
            .filter(|stat_guild| stat_guild.id == id)
            .rev().count() > 1
    }

    pub fn get_guilds(&mut self) -> Vec<Guild> {
        self.guilds
            .clone().into_iter()
            .filter(|stat_guild| stat_guild.available)
            .map(|stat_guild| stat_guild.guild.unwrap())
            .rev().collect()
    }
}