use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum Scope {
    Basic,
    Write,
    OfflineAccess,
}

impl FromStr for Scope {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "basic" => Ok(Scope::Basic),
            "write" => Ok(Scope::Write),
            "offline_access" => Ok(Scope::OfflineAccess),
            _ => Err(anyhow!("not a valid scope!")),
        }
    }
}

#[test]
fn scope() {
    let strings = ["basic", "write", "offline_access"];
    println!("{:#?}", strings.map(|x| x.parse::<Scope>().unwrap()))
}
