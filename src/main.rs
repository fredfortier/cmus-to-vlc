use anyhow::{bail, ensure, Result};
use lazy_static::lazy_static;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

lazy_static! {
    static ref ROOT_PATHS: Vec<&'static str> = vec!["/mnt/media/Torrents"];
}

const TARGET_DIR: &'static str = "/home/user/tmp/test-import";
const PLAYLIST_PATH: &'static str = "/tmp/pl";
const MAX_COMPONENT_LEN: usize = 32;
const TAPER: usize = 10;
const SEPARATOR: &'static str = " - ";

fn main() -> Result<()> {
    println!("Opinionated to import a cmus exported playlist to VLC assuming proper metadata");
    fs::create_dir_all(TARGET_DIR)?;

    let path = Path::new(PLAYLIST_PATH);
    if !path.is_file() {
        bail!("Playlist file not found {}", PLAYLIST_PATH);
    }
    let new_path;
    {
        let mut new_name = path.file_name().unwrap().to_string_lossy().to_string();
        new_name.push_str("~");
        new_path = path.with_file_name(&new_name);
        fs::rename(&path, &new_path)?;
    }
    let file = File::open(&new_path)?;
    for res in BufReader::new(file).lines() {
        let line = res?;
        let source = Path::new(&line);
        if !source.is_file() {
            continue;
        }
        let mut components = vec![];
        let mut ancestors = source.ancestors();
        while let Some(path) = ancestors.next() {
            if components.is_empty() {
                ensure!(path.is_file());
            } else {
                ensure!(!path.is_file());
            }
            if ROOT_PATHS.contains(&path.to_string_lossy().as_ref()) {
                break;
            }
            if let Some(file_name) = path.file_name() {
                components.push((file_name, path.extension()));
            }
        }
        let file_name;
        {
            let mut normalized_components = components
                .into_iter()
                .enumerate()
                .map(|(i, (f, e))| {
                    let mut s = f
                        .to_string_lossy()
                        .replace(SEPARATOR, "-")
                        .trim()
                        .to_string();
                    let t = TAPER * i;
                    let l = if t <= MAX_COMPONENT_LEN - TAPER {
                        MAX_COMPONENT_LEN - t
                    } else {
                        TAPER
                    };
                    if s.len() >= l {
                        s.truncate(l);
                    }
                    s = s.trim().to_string();
                    if let Some(e) = e {
                        s.push_str(".");
                        s.push_str(e.to_string_lossy().as_ref());
                    }
                    s
                })
                .collect::<Vec<_>>();
            normalized_components.reverse();
            file_name = normalized_components.join(SEPARATOR);
        }
        let mut target;
        {
            target = PathBuf::from(TARGET_DIR);
            target.push(file_name);
        }
        if !target.exists() {
            println!("Copying {:?} => {:?}", source, target);
            fs::copy(source, target)?;
        } else {
            println!("Target already exists {:?}", target);
        }
    }
    Ok(())
}
