use std::fmt::Display;

use fred::prelude::*;
use serde::Serialize;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub enum Region {
    /// Africa
    AF,
    /// Antarctica
    AN,
    /// Asia
    AS,
    /// Europe
    EU,
    /// North America
    NA,
    /// Oceania
    OC,
    /// South and Central America
    SA,
    /// Default
    Default,
    /// Globally Available
    World,
    /// Unknown
    Unknown,
}

impl FromRedis for Region {
    fn from_value(value: RedisValue) -> Result<Self, RedisError> {
        let value = value.as_string().ok_or(RedisError::new(
            RedisErrorKind::Parse,
            "Could not convert value to mirror region",
        ))?;
        match value.to_lowercase().as_str() {
            "af" => Ok(Self::AF),
            "an" => Ok(Self::AN),
            "as" => Ok(Self::AS),
            "eu" => Ok(Self::EU),
            "na" => Ok(Self::NA),
            "oc" => Ok(Self::OC),
            "sa" => Ok(Self::SA),
            "default" => Ok(Self::Default),
            "world" => Ok(Self::World),
            _ => Ok(Self::Unknown),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Tier {
    /// Regular mirrors
    Numeric(u64),
    /// Tor mirrors
    #[serde(rename = "tor")]
    Tor,
    #[serde(rename = "unknown")]
    Unknown,
}

impl FromRedis for Tier {
    fn from_value(value: RedisValue) -> Result<Self, RedisError> {
        if let Some(v) = value.as_u64() {
            Ok(Self::Numeric(v))
        } else {
            if let Some(v) = value.as_string() {
                if v.to_lowercase() == "tor" {
                    return Ok(Self::Tor);
                }
            }
            Err(RedisError::new(
                RedisErrorKind::Parse,
                "Could not convert value to mirror tier",
            ))
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub enum Protocol {
    #[serde(rename = "ftp")]
    Ftp,
    #[serde(rename = "http")]
    Http,
    #[serde(rename = "https")]
    Https,
    #[serde(rename = "rsync")]
    Rsync,
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Ftp => "ftp",
            Self::Http => "http",
            Self::Https => "https",
            Self::Rsync => "rsync",
        })
    }
}

impl TryFrom<&str> for Protocol {
    type Error = RedisError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "ftp" => Ok(Self::Ftp),
            "http" => Ok(Self::Http),
            "https" => Ok(Self::Https),
            "rsync" => Ok(Self::Rsync),
            _ => Err(RedisError::new(
                RedisErrorKind::Parse,
                format!("Could not convert {value:?} to protocol"),
            )),
        }
    }
}

impl FromRedis for Protocol {
    fn from_value(value: RedisValue) -> Result<Self, RedisError> {
        let value = value.as_string().ok_or(RedisError::new(
            RedisErrorKind::Parse,
            format!("Could not convert {value:?} to mirror protocol"),
        ))?;
        value.as_str().try_into()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Mirror {
    pub id: u64,
    pub baseurl: String,
    pub region: Region,
    pub location: String,
    pub tier: Tier,
    pub enabled: bool,
    pub protocols: Vec<Protocol>,
}

impl Default for Mirror {
    fn default() -> Self {
        Self {
            id: 0,
            baseurl: "unknown".into(),
            region: Region::Unknown,
            location: "unkonwn".into(),
            tier: Tier::Unknown,
            enabled: false,
            protocols: vec![],
        }
    }
}

impl FromRedis for Mirror {
    fn from_value(value: RedisValue) -> Result<Mirror, RedisError> {
        let value = value.into_map()?;
        let mut mirror = Mirror::default();

        if let Some(v) = value.get(&RedisKey::from_static_str("baseurl")) {
            mirror.baseurl = v.as_string().ok_or(RedisError::new(
                RedisErrorKind::Parse,
                "Could not convert value to mirror (baseurl)",
            ))?;
        } else {
            return Err(RedisError::new(
                RedisErrorKind::Parse,
                "Could not convert value to mirror (baseurl)",
            ));
        }

        if let Some(v) = value.get(&RedisKey::from_static_str("region")) {
            mirror.region = Region::from_value(v.clone())?;
        }

        if let Some(v) = value.get(&RedisKey::from_static_str("location")) {
            mirror.location = v.as_string().ok_or(RedisError::new(
                RedisErrorKind::Parse,
                "Could not convert value to mirror (location)",
            ))?;
        }

        if let Some(v) = value.get(&RedisKey::from_static_str("tier")) {
            mirror.tier = Tier::from_value(v.clone())?;
        }

        if let Some(v) = value.get(&RedisKey::from_static_str("enabled")) {
            mirror.enabled = v.as_u64().map_or(false, |u| u != 0);
        }

        if let Some(v) = value.get(&RedisKey::from_static_str("proto")) {
            mirror.protocols = v
                .as_string()
                .ok_or(RedisError::new(
                    RedisErrorKind::Parse,
                    "Could not convert value to mirror (protocols)",
                ))?
                .split(',')
                .filter_map(|p| p.try_into().ok())
                .collect();
        }

        Ok(mirror)
    }
}
