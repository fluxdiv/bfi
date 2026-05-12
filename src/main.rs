// use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, anyhow};
use quickfig::derive::ConfigFields;
use quickfig::core::{
    Config,
    config_types::JSON,
    VecField
};
use serde::Deserialize;

const TFILE: &'static str = "/home/user/bfi/Cargo.toml";

#[derive(ConfigFields)]
enum AppConfig {
    #[keys("include")]
    Include,
}

#[derive(Deserialize)]
struct Include {
    general: Option<Vec<String>>,
    permissions: Option<Vec<String>>,
    metadata: Option<Vec<String>>,
    size: Option<Vec<String>>,
    count: Option<Vec<String>>,
    access: Option<Vec<String>>
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum CmdOutput {
    GeneralPath(String),
    GeneralType(String),
    PermissionsPermissions(String),
    PermissionsOwner(String),
    PermissionsGroup(String),
    MetaDevice(String),
    MetaInode(String),
    MetaLinks(String),
    SizeB(String),
    SizeKB(String),
    SizeMB(String),
    SizeGB(String),
    SizeTB(String),
    CountLines(String),
    CountWords(String),
    CountBlocks(String),
    AccessRead(String),
    AccessModified(String),
    AccessChanged(String),
    AccessBirth(String)
}



// shelling out to other tools here because
// 1) they work
// 2) they're mostly ubiquitous, with some missing cases handled
// 3) there isn't a benefit to the syscall route for a tool like this
impl Include {
    fn handle_general(&self) {
        if let Some(generals) = &self.general {
            generals.iter().for_each(|o| {
                match o.as_str() {
                    "path" => {
                        // readlink -f filename
                        let out = Command::new("readlink")
                            .arg("-f")
                            .arg(TFILE)
                            .output()
                            .unwrap();

                        let b = String::from_utf8(out.stdout).unwrap();

                        println!("---------");
                        println!("path output");
                        println!("{b}");
                    },
                    "type" => {
                        // file file.name
                        let out = Command::new("file")
                            .arg(TFILE)
                            .output()
                            .unwrap();
                        let b = String::from_utf8(out.stdout).unwrap();
                        println!("---------");
                        println!("file output");
                        println!("{b}");
                    },
                    _ => {/* ignore for now */}
                }
            });
        }
    }
}

fn main() -> Result<()> {
    // /home/user/.config/bfi/config.json
    let config = {
        let mut config_path = dirs::config_dir()
            .ok_or_else(|| {
                anyhow!("Unable to locate home directory")
            })?;
        config_path.push("bfi/config.json");
        Config::<JSON>::open(&config_path)
            .map_err(|_e| anyhow!("Expecting json config at {:#?}", config_path))
    }?;

    let include_field = config.get(AppConfig::Include)
        .ok_or_else(|| anyhow!("bfi config must include \"include\" field"))?;
    include_field.only_one_key()?;

    let include_inner: &serde_json::Value = include_field
        .get_generic_inner()
        .ok_or_else(|| anyhow!("bfi config \"include\" field must be valid json"))?;

    let include: Include = Include::deserialize(include_inner)?;

    include.handle_general();


    Ok(())
}
