use clap::Clap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;

use std::sync::{Arc, Mutex};
use std::{
    error::Error,
    thread::{self, JoinHandle},
};
// use std::fs;
use image::{self, image_dimensions};

use std::path::Path;

#[derive(Clap)]
#[clap(version = "1.0", author = "chai_xb@163.com")]
struct Opts {
    input: Option<String>,
    #[clap(short = 'e', long)]
    regex: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Entity {
    src: String,
    width: u32,
    height: u32,
}

/// 读取文件夹下文件图片的宽高，转换成JSON。
fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    let target_path = &opts.input.unwrap_or(String::from("."));
    let target_path = Path::new(target_path).canonicalize();
    let re = Regex::new(r"(?i)(png|jpeg|jpg)$").unwrap();

    let mut handles = Vec::<JoinHandle<()>>::new();
    let result = Arc::new(Mutex::new(Vec::<Entity>::new()));

    for entry in target_path?.read_dir()? {
        if let Ok(entry) = entry {
            if entry.path().is_file() {
                let shared_result = result.clone();
                // 符合图片后缀
                let filepath = entry.path();
                let extension = filepath
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();
                let inline = || -> () {
                    let handle = thread::spawn(move || {
                        let dimensions = image_dimensions(entry.path()).unwrap();
                        shared_result.lock().unwrap().push(Entity {
                            src: entry.file_name().into_string().unwrap(),
                            width: dimensions.0,
                            height: dimensions.1,
                        });
                    });

                    handles.push(handle);
                };
                if re.is_match(extension) {
                    if let Some(ref include) = opts.regex {
                        let include_regex = Regex::new(include);
                        if include_regex.unwrap().is_match(filepath.to_str().unwrap()) {
                            inline();
                        }
                    } else {
                        inline();
                    }
                }
            }
        }
    }

    for h in handles {
        // Wait
        h.join().unwrap();
    }
    let result = result.lock().unwrap();

    // WTF
    // println!("{:?}", serde_json::to_string_pretty(&*result).unwrap());
    println!("{}", serde_json::json!(*result));

    Ok(())
}
