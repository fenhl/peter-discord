//! The base library for the Gefolge Discord bot, Peter

#![cfg_attr(test, deny(warnings))]
#![warn(trivial_casts)]
#![deny(unused, missing_docs, unused_qualifications)]
#![forbid(unused_import_braces)]

use std::{
    env,
    fmt,
    io::{
        self,
        BufReader,
        prelude::*
    },
    net::TcpStream,
    sync::Arc
};
use serenity::{
    client::bridge::gateway::ShardManager,
    model::prelude::*,
    prelude::*
};
use typemap::Key;
use wrapped_enum::wrapped_enum;

pub mod bitbar;
pub mod commands;
pub mod emoji;
pub mod lang;
pub mod model;
pub mod parse;
pub mod user_list;
pub mod werewolf;

/// The Gefolge guild's ID.
pub const GEFOLGE: GuildId = GuildId(355761290809180170);

/// The address and port where the bot listens for IPC commands.
pub const IPC_ADDR: &str = "127.0.0.1:18807";

/// A collection of possible errors not simply forwarded from other libraries.
#[derive(Debug)]
pub enum OtherError {
    /// Returned if a Serenity context was required outside of an event handler but the `ready` event has not been received yet.
    MissingContext,
    /// Returned by the user list handler if a user has no join date.
    MissingJoinDate,
    /// The reply to an IPC command did not end in a newline.
    MissingNewline,
    /// Returned from `listen_ipc` if a command line was not valid shell lexer tokens.
    Shlex,
    /// Returned from `listen_ipc` if an unknown command is received.
    UnknownCommand(Vec<String>)
}

impl fmt::Display for OtherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            OtherError::MissingContext => write!(f, "Serenity context not available before ready event"),
            OtherError::MissingJoinDate => write!(f, "encountered user without join date"),
            OtherError::MissingNewline => write!(f, "the reply to an IPC command did not end in a newline"),
            OtherError::Shlex => write!(f, "failed to parse IPC command line"),
            OtherError::UnknownCommand(ref args) => write!(f, "unknown command: {:?}", args)
        }
    }
}

wrapped_enum! {
    #[allow(missing_docs)]
    #[derive(Debug)]
    pub enum Error {
        #[allow(missing_docs)]
        ChannelIdParse(ChannelIdParseError),
        #[allow(missing_docs)]
        Env(env::VarError),
        #[allow(missing_docs)]
        GameAction(String),
        #[allow(missing_docs)]
        Io(io::Error),
        #[allow(missing_docs)]
        Json(serde_json::Error),
        #[allow(missing_docs)]
        Other(OtherError),
        #[allow(missing_docs)]
        QwwStartGame(quantum_werewolf::game::state::StartGameError),
        #[allow(missing_docs)]
        RoleIdParse(RoleIdParseError),
        #[allow(missing_docs)]
        Serenity(serenity::Error),
        #[allow(missing_docs)]
        UserIdParse(UserIdParseError),
        #[allow(missing_docs)]
        Wrapped((String, Box<Error>))
    }
}

/// A helper trait for annotating errors with more informative error messages.
pub trait IntoResult<T> {
    /// Annotates an error with an additional message which is displayed along with the error.
    fn annotate(self, msg: impl Into<String>) -> Result<T>;
}

impl<T, E: Into<Error>> IntoResult<T> for std::result::Result<T, E> {
    fn annotate(self, msg: impl Into<String>) -> Result<T> {
        self.map_err(|e| Error::Wrapped((msg.into(), Box::new(e.into()))))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ChannelIdParse(ref e) => e.fmt(f),
            Error::Env(ref e) => e.fmt(f),
            Error::GameAction(ref s) => write!(f, "invalid game action: {}", s),
            Error::Io(ref e) => e.fmt(f),
            Error::Json(ref e) => e.fmt(f),
            Error::Other(ref e) => e.fmt(f),
            Error::QwwStartGame(ref e) => e.fmt(f),
            Error::RoleIdParse(ref e) => e.fmt(f),
            Error::Serenity(ref e) => e.fmt(f),
            Error::UserIdParse(ref e) => e.fmt(f),
            Error::Wrapped((ref msg, ref e)) => write!(f, "{}: {}", msg, e)
        }
    }
}

impl std::error::Error for Error {}

#[allow(missing_docs)]
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// `typemap` key for the serenity shard manager.
pub struct ShardManagerContainer;

impl Key for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

/// Sends an IPC command to the bot.
///
/// **TODO:** document available IPC commands
pub fn send_ipc_command<T: fmt::Display, I: IntoIterator<Item = T>>(cmd: I) -> Result<String, Error> {
    let mut stream = TcpStream::connect(IPC_ADDR)?;
    writeln!(&mut stream, "{}", cmd.into_iter().map(|arg| shlex::quote(&arg.to_string()).into_owned()).collect::<Vec<_>>().join(" "))?;
    let mut buf = String::default();
    BufReader::new(stream).read_line(&mut buf)?;
    if buf.pop() != Some('\n') { return Err(OtherError::MissingNewline.into()) }
    Ok(buf)
}

/// Utility function to shut down all shards.
pub fn shut_down(ctx: &Context) {
    ctx.invisible(); // hack to prevent the bot showing as online when it's not
    let data = ctx.data.lock();
    let mut shard_manager = data.get::<ShardManagerContainer>().expect("missing shard manager").lock();
    shard_manager.shutdown_all();
}
