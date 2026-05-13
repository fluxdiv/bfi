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

// Helpers for converting bytes to others
fn bytes_to_b(bytes: u64) -> String {
    format!("{:.2} B", bytes as f64)
}

fn bytes_to_kb(bytes: u64) -> String {
    format!("{:.2} KB", bytes as f64 / 1024.0)
}

fn bytes_to_mb(bytes: u64) -> String {
    format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0)
}

fn bytes_to_gb(bytes: u64) -> String {
    format!("{:.2} GB", bytes as f64 / 1024.0 / 1024.0 / 1024.0)
}

fn bytes_to_tb(bytes: u64) -> String {
    format!("{:.2} TB", bytes as f64 / 1024.0 / 1024.0 / 1024.0 / 1024.0)
}

// shelling out to other tools here because
// 1) they work
// 2) they're mostly ubiquitous
// 3) there isn't a benefit to the syscall route for a tool like this
impl Include {
    fn handle_general(
        &self,
        heap: Arc<Mutex<BinaryHeap<Reverse<CmdOutput>>>>
    ) -> Vec<JoinHandle<()>> {
        let mut threads: Vec<JoinHandle<()>> = Vec::with_capacity(2);
        if let Some(generals) = &self.general {
            generals.iter().for_each(|o| {
                match o.as_str() {
                    "path" => {
                        let heap = Arc::clone(&heap);
                        threads.push(thread::spawn(move || {
                            let out = Command::new("readlink")
                                .arg("-f")
                                .arg(TFILE)
                                .output();

                            match out {
                                Err(e) => {
                                    eprintln!("{e}");
                                },
                                Ok(out) => {
                                    let b = String::from_utf8(out.stdout).unwrap();
                                    let cmdout = CmdOutput::GeneralPath(b.trim().to_string());

                                    let mut heaplock = heap.lock().unwrap();
                                    heaplock.push(Reverse(cmdout));
                                },
                            }

                        }));
                    },
                    "type" => {
                        let heap = Arc::clone(&heap);
                        threads.push(thread::spawn(move || {
                            let out = Command::new("file")
                                .arg(TFILE)
                                .output();

                            match out {
                                Err(e) => {
                                    eprintln!("{e}");
                                },
                                Ok(out) => {
                                    let b = String::from_utf8(out.stdout).unwrap();
                                    let cmdout = CmdOutput::GeneralType(b.trim().to_string());

                                    let mut heaplock = heap.lock().unwrap();
                                    heaplock.push(Reverse(cmdout));
                                },
                            }
                        }));
                    },
                    e => {
                        eprintln!("Unrecognized option in \"general\" field: {e}");
                        eprintln!("Available options: path, type");
                    }
                }
            });
        }
        threads
    }


    fn handle_size(
        &self,
        heap: Arc<Mutex<BinaryHeap<Reverse<CmdOutput>>>>
    ) -> Vec<JoinHandle<()>> {
        let mut threads: Vec<JoinHandle<()>> = Vec::with_capacity(5);

        if let Some(sizes) = &self.size && sizes.len() > 0 {
            let heap = Arc::clone(&heap);
            let sizes = sizes.clone();
            threads.push(thread::spawn(move || {
                let out = Command::new("du")
                    .arg("-sb")
                    .arg(TFILE)
                    .output();

                if let Err(e) = out {
                    eprintln!("{e}");
                    return;
                }

                let b = String::from_utf8(out.unwrap().stdout).unwrap();
                let size_bytes_res = b
                    .split_whitespace()
                    .next()
                    .unwrap()
                    .to_string()
                    .parse::<u64>();

                if size_bytes_res.is_err() {
                    eprintln!("du -sb returned unparseable");
                    return;
                }
                let size_bytes = size_bytes_res.unwrap();

                sizes.iter().take(5).for_each(|s| {
                    match s.to_ascii_uppercase().as_str() {
                        "B" => {
                            let mut heaplock = heap.lock().unwrap();
                            heaplock.push(
                                Reverse(CmdOutput::SizeB(bytes_to_b(size_bytes)))
                            );
                        }
                        "KB" => {
                            let mut heaplock = heap.lock().unwrap();
                            heaplock.push(
                                Reverse(CmdOutput::SizeKB(bytes_to_kb(size_bytes)))
                            );
                        }
                        "MB" => {
                            let mut heaplock = heap.lock().unwrap();
                            heaplock.push(
                                Reverse(CmdOutput::SizeMB(bytes_to_mb(size_bytes)))
                            );
                        }
                        "GB" => {
                            let mut heaplock = heap.lock().unwrap();
                            heaplock.push(
                                Reverse(CmdOutput::SizeGB(bytes_to_gb(size_bytes)))
                            );
                        }
                        "TB" => {
                            let mut heaplock = heap.lock().unwrap();
                            heaplock.push(
                                Reverse(CmdOutput::SizeTB(bytes_to_tb(size_bytes)))
                            );
                        }
                        e => {
                            eprintln!("Unrecognized input in \"sizes\" config field: {e}");
                            eprintln!("Available sizes: B, KB, MB, GB, TB");
                        }
                    }
                });
            }));
        }

        threads
    }

    fn handle_count(
        &self,
        heap: Arc<Mutex<BinaryHeap<Reverse<CmdOutput>>>>
    ) -> Vec<JoinHandle<()>> {
        let mut threads: Vec<JoinHandle<()>> = Vec::with_capacity(3);
        if let Some(counts) = &self.count && counts.len() > 0 {
            // looping multiple times is irrelevant
            // the longest this will ever be is 3 
            let haslines = counts.contains(&"lines".to_string());
            let haswords = counts.contains(&"words".to_string());
            if counts.iter().any(|c| c.eq("lines") || c.eq("words")) {
                let heap = Arc::clone(&heap);
                threads.push(thread::spawn(move || {
                    let out = Command::new("wc")
                        .arg("-l").arg("-w")
                        .arg(TFILE)
                        .output();

                    if let Err(e) = out {
                        eprintln!("{e}");
                        return;
                    }

                    let b = String::from_utf8(out.unwrap().stdout).unwrap();
                    let outs: Vec<&str> = b
                        .trim()
                        .split_whitespace()
                        .collect();

                    // 0 is lines, 1 is words
                    let lines = outs.get(0).unwrap();
                    let words = outs.get(1).unwrap();
                    // only take lock once depending on combination
                    if haslines && haswords {
                        let linesout = CmdOutput::CountLines(lines.to_string());
                        let wordsout = CmdOutput::CountWords(words.to_string());
                        let mut heaplock = heap.lock().unwrap();
                        heaplock.extend(vec![Reverse(linesout), Reverse(wordsout)]);
                    } else if haslines {
                        let linesout = CmdOutput::CountLines(lines.to_string());
                        let mut heaplock = heap.lock().unwrap();
                        heaplock.push(Reverse(linesout));
                    } else {
                        let wordsout = CmdOutput::CountWords(words.to_string());
                        let mut heaplock = heap.lock().unwrap();
                        heaplock.push(Reverse(wordsout));
                    }
                }));
            }

            counts.iter().for_each(|c| {
                match c.as_str() {
                    "lines" | "words" => {}
                    "blocks" => {
                        let heap = Arc::clone(&heap);
                        threads.push(thread::spawn(move || {
                            let out = Command::new("stat")
                                .arg("-c")
                                .arg("%b")
                                .arg(TFILE)
                                .output();
                            if let Err(e) = out {
                                eprintln!("{e}");
                                return;
                            }
                            let b = String::from_utf8(out.unwrap().stdout).unwrap();
                            let b = b.trim().to_string();
                            let cmdout = CmdOutput::CountBlocks(b);
                            let mut heaplock = heap.lock().unwrap();
                            heaplock.push(Reverse(cmdout));
                        }));
                    }
                    e => {
                        eprintln!("Unrecognized option in \"counts\" field: {e}");
                        eprintln!("Available options: words, lines, blocks");
                    }
                }
            });
        }
        threads
    }

    fn handle_perms_meta_access(
        &self,
        heap: Arc<Mutex<BinaryHeap<Reverse<CmdOutput>>>>
    ) -> Vec<JoinHandle<()>> {
        let mut threads: Vec<JoinHandle<()>> = vec![];
        let perms = self.permissions.clone().unwrap_or_default();
        let meta = self.metadata.clone().unwrap_or_default();
        let access = self.access.clone().unwrap_or_default();
        let opts = perms.into_iter().chain(meta).chain(access);
        let heap = Arc::clone(&heap);

        threads.push(thread::spawn(move || {
            let out = Command::new("stat")
                .arg("-c")
                .arg("'%A|%U %u|%G %g|%D|%i|%h|%x|%y|%z|%w'")
                .arg(TFILE)
                .output();
            if let Err(e) = out {
                eprintln!("{e}");
                return;
            }
            let b = String::from_utf8(out.unwrap().stdout).unwrap();
            let outs: Vec<&str> = b
                .trim()
                .trim_matches('\'')
                .split('|')
                .collect();

            opts.into_iter().for_each(|p| {
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
                    "device" => {
                        let out = outs
                            .get(3)
                            .unwrap_or_else(|| &"")
                            .to_string();
                        let cmdout = CmdOutput::MetaDevice(out);
                        let mut heaplock = heap.lock().unwrap();
                        heaplock.push(Reverse(cmdout));
                    },
                    "inode" => {
                        let out = outs
                            .get(4)
                            .unwrap_or_else(|| &"")
                            .to_string();
                        let cmdout = CmdOutput::MetaInode(out);
                        let mut heaplock = heap.lock().unwrap();
                        heaplock.push(Reverse(cmdout));
                    },
                    "links" => {
                        let out = outs
                            .get(5)
                            .unwrap_or_else(|| &"")
                            .to_string();
                        let cmdout = CmdOutput::MetaLinks(out);
                        let mut heaplock = heap.lock().unwrap();
                        heaplock.push(Reverse(cmdout));
                    },
                    "read" => {
                        let out = outs
                            .get(6)
                            .unwrap_or_else(|| &"")
                            .to_string();
                        let cmdout = CmdOutput::AccessRead(out);
                        let mut heaplock = heap.lock().unwrap();
                        heaplock.push(Reverse(cmdout));
                    },
                    "modified" => {
                        let out = outs
                            .get(7)
                            .unwrap_or_else(|| &"")
                            .to_string();
                        let cmdout = CmdOutput::AccessModified(out);
                        let mut heaplock = heap.lock().unwrap();
                        heaplock.push(Reverse(cmdout));
                    },
                    "changed" => {
                        let out = outs
                            .get(8)
                            .unwrap_or_else(|| &"")
                            .to_string();
                        let cmdout = CmdOutput::AccessChanged(out);
                        let mut heaplock = heap.lock().unwrap();
                        heaplock.push(Reverse(cmdout));
                    },
                    "birth" => {
                        let out = outs
                            .get(9)
                            .unwrap_or_else(|| &"")
                            .to_string();
                        let cmdout = CmdOutput::AccessBirth(out);
                        let mut heaplock = heap.lock().unwrap();
                        heaplock.push(Reverse(cmdout));
                    },
                    e => {
                        eprintln!("Unrecognized option in config: {e}");
                        eprintln!("Available \"permissions\" options: permissions, owner, group");
                        eprintln!("Available \"metadata\" options: device, inode, links");
                        eprintln!("Available \"access\" options: birth, read, modified, changed");
                    }
                }
            });
        }));

        threads
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

    let mut heap = Arc::new(
        Mutex::new(BinaryHeap::<Reverse<CmdOutput>>::with_capacity(50))
    );

    let mut threads: Vec<JoinHandle<()>> = vec![];

    threads
        .extend(include.handle_general(Arc::clone(&heap)));
    threads
        .extend(include.handle_perms_meta_access(Arc::clone(&heap)));
    threads
        .extend(include.handle_count(Arc::clone(&heap)));
    threads
        .extend(include.handle_size(Arc::clone(&heap)));

    threads.into_iter().for_each(|t| t.join().unwrap());

    let mut heap = heap.lock().unwrap();

    // while let Some(v) = heap.pop() {
    //     println!("{:?}", v.0);
    // }

    Ok(())
}
