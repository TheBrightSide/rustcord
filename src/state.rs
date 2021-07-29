use twilight_model::id::GuildId;
use twilight_model::user::CurrentUser;
use twilight_model::guild::Guild;
use twilight_model::channel::Channel;
use twilight_model::channel::Message;
use twilight_gateway::Event;

#[derive(Clone)]
pub struct ClientState {
    authenticated: bool,
    guild_ids: Vec<GuildId>,
    user: Option<CurrentUser>,
    current_guild: Option<Guild>,
    current_channel: Option<Channel>,
    messages: Vec<Message>
}

impl ClientState {
    pub const fn new() -> Self {
        ClientState {
            authenticated: false,
            guild_ids: vec![],
            user: None,
            current_guild: None,
            current_channel: None,
            messages: vec![]
        }
    }

    pub fn process_event(&mut self, event: Event) {
        match event {
            Event::Ready(data) => {
                self.authenticated = true;
                self.user = Option::from(data.user);
                self.guild_ids = data.guilds.into_iter().map(
                    |guild| guild.id
                ).rev().collect();
            },
            _ => {}
        }
    }
}