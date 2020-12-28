use {
    std::collections::{
        BTreeMap,
        BTreeSet,
    },
    serde::{
        Deserialize,
        Serialize,
    },
    serenity::{
        model::prelude::*,
        prelude::*,
    },
    tokio::{
        fs::File,
        prelude::*,
    },
    crate::{
        Error,
        twitch,
        werewolf,
    },
};

const PATH: &str = "/usr/local/share/fidera/config.json";

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub channels: Channels,
    pub peter: Peter,
    pub(crate) twitch: twitch::Config,
    pub werewolf: BTreeMap<GuildId, werewolf::Config>,
}

impl TypeMapKey for Config {
    type Value = Config;
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Channels {
    pub ignored: BTreeSet<ChannelId>,
    pub voice: ChannelId,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Peter {
    pub bot_token: String,
    pub(crate) self_assignable_roles: BTreeSet<RoleId>,
}

impl Config {
    pub async fn new() -> Result<Config, Error> {
        let mut file = File::open(PATH).await?;
        let mut buf = String::default();
        file.read_to_string(&mut buf).await?;
        Ok(serde_json::from_str(&buf)?) //TODO use async-json
    }

    /*
    pub(crate) async fn save(self) -> Result<(), Error> {
        let buf = serde_json::to_vec(&self)?; //TODO use async-json
        File::create(PATH).await?.write_all(&buf).await?;
        Ok(())
    }
    */
}
