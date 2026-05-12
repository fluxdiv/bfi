use std::cmp::Reverse;
use std::collections::BinaryHeap;
// use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    fn handle_general(
        &self,
        heap: Arc<Mutex<BinaryHeap<Reverse<CmdOutput>>>>
    ) -> Vec<JoinHandle<()>> {
        if let Some(generals) = &self.general {
            let threads: Vec<JoinHandle<()>> = generals.iter().map(|o| {
                match o.as_str() {
                    "path" => {
                        let heap = Arc::clone(&heap);
                        thread::spawn(move || {
                            // readlink -f filename
                            let out = Command::new("readlink")
                                .arg("-f")
                                .arg(TFILE)
                                .output()
                                .unwrap();

                            let b = String::from_utf8(out.stdout).unwrap();
                            let cmdout = CmdOutput::GeneralPath(b.trim().to_string());

                            let mut heaplock = heap.lock().unwrap();
                            heaplock.push(Reverse(cmdout));
                        })
                    },
                    "type" => {
                        let heap = Arc::clone(&heap);
                        thread::spawn(move || {
                            // file file.name
                            let out = Command::new("file")
                                .arg(TFILE)
                                .output()
                                .unwrap();
                            let b = String::from_utf8(out.stdout).unwrap();
                            let cmdout = CmdOutput::GeneralType(b.trim().to_string());

                            let mut heaplock = heap.lock().unwrap();
                            heaplock.push(Reverse(cmdout));
                        })
                    },
                    _ => todo!()
                }
            }).collect();
            return threads;
        }
        vec![]
    }

    fn handle_permissions(
        &self,
        heap: Arc<Mutex<BinaryHeap<Reverse<CmdOutput>>>>
    ) -> Vec<JoinHandle<()>> {
        let mut ret: Vec<JoinHandle<()>> = vec![];
        if let Some(permissions) = &self.permissions {
            // get all regardless via 1 command/thread,
            // but only keep what the user asked for
            if permissions.len() > 0 {
                let heap = Arc::clone(&heap);
                let permissions = permissions.clone();
                ret.push(thread::spawn(move || {
                    let out = Command::new("stat")
                        .arg("-c")
                        .arg("'%A|%U %u|%G %g'")
                        .arg(TFILE)
                        .output()
                        .unwrap();
                    let b = String::from_utf8(out.stdout).unwrap();
                    let outs: Vec<&str> = b
                        .trim()
                        .trim_matches('\'')
                        .split('|')
                        .collect();
                    println!("{:#?}", outs);

                    permissions.iter().for_each(|p| {
                        match p.as_str() {
                            "permissions" => {
                                let out = outs
                                    .get(0)
                                    .unwrap_or_else(|| &"")
                                    .to_string();
                                let cmdout = CmdOutput::PermissionsPermissions(out);
                                let mut heaplock = heap.lock().unwrap();
                                heaplock.push(Reverse(cmdout));
                            },
                            "owner" => {
                                let out = outs
                                    .get(1)
                                    .unwrap_or_else(|| &"")
                                    .to_string();
                                let cmdout = CmdOutput::PermissionsOwner(out);
                                let mut heaplock = heap.lock().unwrap();
                                heaplock.push(Reverse(cmdout));
                            },
                            "group" => {
                                let out = outs
                                    .get(2)
                                    .unwrap_or_else(|| &"")
                                    .to_string();
                                let cmdout = CmdOutput::PermissionsGroup(out);
                                let mut heaplock = heap.lock().unwrap();
                                heaplock.push(Reverse(cmdout));
                            },
                            _ => todo!()
                        }
                    });
                }));
            }
        }
        ret
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

    let mut heap = Arc::new(Mutex::new(BinaryHeap::<Reverse<CmdOutput>>::new()));

    let mut threads: Vec<JoinHandle<()>> = vec![];

    threads
        .extend(include.handle_general(Arc::clone(&heap)));
    threads
        .extend(include.handle_permissions(Arc::clone(&heap)));

    threads.into_iter().for_each(|t| t.join().unwrap());

    let mut heap = heap.lock().unwrap();

    while let Some(v) = heap.pop() {
        println!("{:?}", v.0);
    }

    Ok(())
}
