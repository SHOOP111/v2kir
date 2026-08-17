use std::{fs, path::PathBuf};
use crate::model::SaveData;

fn path() -> PathBuf {
    let base = std::env::var_os("AETHERFALL_SAVE_DIR").map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")) .join("save"));
    base.join("profile.json")
}

pub fn load() -> SaveData {
    let p=path();
    let Ok(raw)=fs::read_to_string(p) else { return SaveData::default(); };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn store(save:&SaveData){
    let p=path(); if let Some(parent)=p.parent(){let _=fs::create_dir_all(parent);}
    if let Ok(raw)=serde_json::to_string_pretty(save){let _=fs::write(p,raw);}
}
