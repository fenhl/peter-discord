//! Some utilities for working with emoji (both Unicode and custom) and message reactions.

use std::{fmt, fs, io, mem};
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::str::FromStr;

use regex::Regex;

use serenity::model::{EmojiId, ReactionType};

/// An error that can occur while parsing emoji from a message.
#[derive(Debug)]
pub enum Error {
    /// An error occurred while decoding a filename.
    FilenameDecode(OsString),
    /// A `std::io::Error` occurred.
    Io(io::Error)
}

impl From<OsString> for Error {
    fn from(e: OsString) -> Error {
        Error::FilenameDecode(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::FilenameDecode(ref s) => write!(f, "failed to read twemoji filename: {:?}", s),
            Error::Io(ref e) => write!(f, "io error while building emoji db: {}", e)
        }
    }
}

/// An iterator over all the emoji in a message.
pub struct Iter {
    text: String,
    emoji: Vec<String>
}

impl Iter {
    /// Create an iterator over all emoji in the given text.
    pub fn new(text: String) -> Result<Iter, Error> {
        lazy_static! {
            static ref RE: Regex = Regex::new("^([0-9a-f]{1,6}(?:-[0-9a-f]{1,6})*)\\.svg$").expect("failed to compile twemoji filename regex");
        }
        let mut emoji = BTreeSet::default();
        for entry in fs::read_dir("/opt/git/github.com/twitter/twemoji/master/2/svg")? {
            let file_name = entry?.file_name().into_string()?;
            if let Some(capture) = RE.captures(&file_name).and_then(|captures| captures.get(1)) {
                // convert the filename encoding the emoji (e.g. 1f3f3-fe0f-200d-1f308.svg) to the emoji itself (e.g. 🏳️‍🌈)
                emoji.insert(capture.as_str().split('-').filter_map(|hex| u32::from_str_radix(hex, 16).ok().and_then(::std::char::from_u32)).collect());
            }
        }
        Ok(Iter {
            text,
            emoji: emoji.into_iter().collect()
        })
    }
}

impl Iterator for Iter {
    type Item = ReactionType;

    fn next(&mut self) -> Option<ReactionType> {
        lazy_static! {
            static ref RE: Regex = Regex::new("^<:[0-9A-Z_a-z]{2,}:[0-9]+>").expect("failed to compile custom emoji regex");
        }
        let text = mem::replace(&mut self.text, String::default());
        let mut text = &text[..];
        loop {
            if let Some(captures) = RE.captures(text) {
                let capture = captures.get(0).expect("failed to capture match object").as_str();
                if let Some(emoji) = parse_custom_emoji(capture) {
                    self.text = text[capture.len()..].to_owned();
                    break Some(emoji);
                }
            }
            if let Some(emoji) = self.emoji.iter().rev().filter(|&emoji| text.starts_with(emoji)).next() { // longest emoji first
                self.text = text[emoji.len()..].to_owned();
                break Some(ReactionType::Unicode(emoji.to_owned()));
            }
            match text.char_indices().nth(1) {
                Some((idx, _)) => { text = &text[idx..]; }
                None => { break None; }
            }
        }
    }
}

/// Takes a custom emoji in the format of `<:lrrJUDGE:289173939802996736>` and returns it as a Serenity `ReactionType`.
pub fn parse_custom_emoji(text: &str) -> Option<ReactionType> {
    lazy_static! {
        static ref RE: Regex = Regex::new("^<:([0-9A-Z_a-z]{2,}):([0-9]+)>$").expect("failed to compile custom emoji regex");
    }
    RE.captures(text)
        .and_then(|captures| match (captures.get(1), captures.get(2)) {
            (Some(name), Some(id)) => Some((name.as_str(), id.as_str())),
            _ => None
        })
        .and_then(|(name, id)| Some(ReactionType::Custom {
            id: EmojiId(u64::from_str(id).expect("failed to parse custom emoji ID")),
            name: Some(name.to_owned())
        }))
}
