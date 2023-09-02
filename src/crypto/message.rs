use anyhow::{Context, Error};
use base64::{engine::general_purpose, Engine as _};
use lazy_static::lazy_static;
use regex::Regex;
use std::{fmt, str::FromStr};

use super::keys::{Key, Nonce};

lazy_static! {
    /// The encoder to use when serializing/deserializing the message.
    static ref ENCODER: base64::engine::GeneralPurpose = general_purpose::STANDARD;

    /// A pattern matching stored strings in EJSON format. Which is:
    /// EJ[<version>:<base64 key>:<base64 nonce>:<base64 encrypted value>]
    static ref PATTERN: Regex =
        Regex::new(r"^EJ\[\d:[A-Za-z0-9+=/]{44}:[A-Za-z0-9+=/]{32}:(.+)\]$").unwrap();
}

/// A struct representing an encrypted message. This is what is stored in the encrypted field
/// in EJSON files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Message {
    pub version: u8,
    pub key: Key,
    pub nonce: Nonce,
    pub value: Vec<u8>,
}

impl Message {
    /// Returns whether or not the supplied string is an EJSON encoded message.
    pub fn is_valid(encoded_str: &str) -> bool {
        PATTERN.is_match(encoded_str)
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EJ[{}:{}:{}:{}]",
            self.version,
            ENCODER.encode(self.key.0),
            ENCODER.encode(self.nonce.0),
            ENCODER.encode(self.value.as_slice()),
        )
    }
}

impl FromStr for Message {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !Self::is_valid(s) {
            return Err(anyhow::anyhow!("Oops"));
        }

        let key = ENCODER.decode(&s[5..49]).context("decoding key")?;
        let nonce = ENCODER.decode(&s[50..82]).context("decoding nonce")?;
        let value = ENCODER
            .decode(&s[83..s.len() - 1])
            .context("decoding value")?;

        Ok(Self {
            version: s[3..4].parse()?,
            key: Key(key.as_slice().try_into()?),
            nonce: Nonce(nonce.as_slice().try_into()?),
            value,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const VALID_KEY: &str =
        "EJ[1:AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=:AgICAgICAgICAgICAgICAgICAgICAgIC:AwMD]";

    #[test]
    fn is_valid() {
        let cases = [(VALID_KEY, true), ("nope", false), ("EJ[]", false)];

        cases.into_iter().for_each(|(given, exp)| {
            assert_eq!(exp, Message::is_valid(given), "{} != {}", given, exp);
        });
    }

    #[test]
    fn display() {
        let msg = Message {
            version: 1,
            key: Key::all(1),
            nonce: Nonce::all(2),
            value: vec![3, 3, 3],
        };

        assert_eq!(VALID_KEY, msg.to_string());
    }

    #[test]
    fn from_str() {
        assert!("nope".parse::<Message>().is_err());

        let parsed: Message = VALID_KEY.parse().unwrap();

        let exp = Message {
            version: 1,
            key: Key::all(1),
            nonce: Nonce::all(2),
            value: vec![3, 3, 3],
        };

        assert_eq!(exp, parsed);
    }
}
