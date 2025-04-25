use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

type Dict = HashMap<String, String>;

fn parse_dict_from_str(data: &str) -> Dict {
    data.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let word = parts[0].to_lowercase();
                let phonetic = parts[1..].join(" ");
                Some((word, phonetic))
            } else {
                None
            }
        })
        .collect()
}

fn parse_dict_from_bytes(bytes: &[u8], replacement: &str) -> Dict {
    let content = String::from_utf8_lossy(bytes).replace('�', replacement);
    parse_dict_from_str(&content)
}

macro_rules! load_dict {
    ($name:ident, $path:expr) => {
        static $name: Lazy<Mutex<Dict>> = Lazy::new(|| {
            //let dict = parse_dict_from_str(include_str!($path));
             let dict = parse_dict_from_bytes(include_bytes!($path), "🤖");
            Mutex::new(dict)
        });
    };
}

load_dict!(IPA_DICT, "en_UK.txt");
load_dict!(CMU_SIMPLE_DICT, "cmudict_simple.txt");

// static CMU_SIMPLE_DICT: Lazy<Mutex<Dict>> = Lazy::new(|| {
//     let dict = parse_dict_from_bytes(include_bytes!("cmudict_simple.txt"), "🤖");
//     Mutex::new(dict)
// });


#[derive(Deserialize)]
struct RequestData {
    text: String,
}

#[derive(Serialize, Debug)]
pub struct Pair {
    text: String,
    phonetic: String,
}

#[derive(Serialize, Debug)]
pub struct ResponseData {
    phonetic: Vec<Pair>,
    phonetic_sentence: Option<String>
}

pub struct PhoneticConverter;

impl PhoneticConverter {
    pub fn new() -> Self {
        Self
    }

    fn get_dicts(&self) -> (std::sync::MutexGuard<Dict>, std::sync::MutexGuard<Dict>) {
        (IPA_DICT.lock().unwrap(), CMU_SIMPLE_DICT.lock().unwrap())
    }

    pub fn convert_sentence(&self, input: &str) -> Vec<String> {
        let (ipa_dict, cmu_simple_dict) = self.get_dicts();

        input
            .split_whitespace()
            .map(|word| {
                let key = word.to_lowercase().replace(".", "");
                cmu_simple_dict
                    .get(&key)
                    .or_else(|| ipa_dict.get(&key))
                    .cloned()
                    .unwrap_or_else(|| word.to_string())
            })
            .collect()
    }

    pub fn convert(&self, input: &str) -> ResponseData {
        let (ipa_dict, cmu_simple_dict) = self.get_dicts();

        let phonetic = input
            .split_whitespace()
            .map(|word| {
                let key = word.to_lowercase().replace(".", "");
                let phonetic = cmu_simple_dict
                    .get(&key)
                    .or_else(|| ipa_dict.get(&key))
                    .cloned()
                    .unwrap_or_else(|| word.to_string());

                Pair {
                    text: word.to_string(),
                    phonetic,
                }
            })
            .collect();

        ResponseData { phonetic, phonetic_sentence:None }
    }
}

fn main() {
    
    let text = "software like Aldus PageMaker including versions of Lorem Ipsum.".to_string();
    let converter = PhoneticConverter::new();

    let res = converter.convert("always");
    let sent = converter.convert_sentence(&text);

    println!("{:?}", res);
    println!("{:?}", sent);
}
