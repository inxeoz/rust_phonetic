
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

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


static IPA_DICT: Lazy<Dict> = Lazy::new(|| parse_dict_from_bytes(include_bytes!("en_UK.txt"), " "));

static CMU_SIMPLE_DICT: Lazy<Dict> =
    Lazy::new(|| parse_dict_from_bytes(include_bytes!("cmudict_simple.txt"), " "));

#[derive(Deserialize, Serialize, Debug)]
#[wasm_bindgen]
pub enum ClientRequestType {
    MapRes,
    SentRes,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum PhoneticResponceType {
    MapRes(Vec<Pair>),
    SentRes(Vec<String>),
}
#[derive(Deserialize, Serialize, Debug)]
#[wasm_bindgen]
pub struct RequestData {
    text: String,
    operation: ClientRequestType,
}

#[derive(Deserialize, Serialize, Debug)]
#[wasm_bindgen]
pub struct Pair {
    text: String,
    phonetic: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[wasm_bindgen]
pub struct ResponseData {
    phonetic: PhoneticResponceType,
}

static  CONVERTER_INSTANCE: Lazy<PhoneticConverter> = Lazy::new(||PhoneticConverter::new());
pub struct PhoneticConverter;

impl PhoneticConverter {
    pub fn new() -> Self {
        Self
    }

    fn get_dicts(&self) -> (&'static Dict, &'static Dict) {
        (&IPA_DICT, &CMU_SIMPLE_DICT)
    }

    pub fn convert_sentence(&self, input: &str) -> Vec<String> {
        let (ipa_dict, cmu_simple_dict) = self.get_dicts();

        let phonetic = input
            .split_whitespace()
            .map(|word| {
                let key = word.to_lowercase().replace(".", "");
                cmu_simple_dict
                    .get(&key)
                    .or_else(|| ipa_dict.get(&key))
                    .cloned()
                    .unwrap_or_else(|| word.to_string())
            })
            .collect();
        phonetic
    }

    pub fn convert(&self, input: &str) -> Vec<Pair> {
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

        phonetic
    }
}

pub fn ConvertMainInner(req: &RequestData) -> ResponseData {
    let converter = &CONVERTER_INSTANCE;
    let response = match req.operation {
        ClientRequestType::MapRes => {
            let res = converter.convert(&req.text);
            ResponseData { phonetic: PhoneticResponceType::MapRes(res) }
        }
        ClientRequestType::SentRes => {
            let res = converter.convert_sentence(&req.text);
            ResponseData {
                phonetic: PhoneticResponceType::SentRes(res),
            }
        }
    };

    response
}
#[wasm_bindgen]
pub fn ConvertMain(req_js: JsValue) -> Result<JsValue, JsValue> {
    let req : RequestData = serde_wasm_bindgen::from_value(req_js)?;
    let response = ConvertMainInner(&req);
    Ok(serde_wasm_bindgen::to_value(&response)?)
}

use serde_wasm_bindgen::Error;
fn main() -> Result<(), Error> {
    Ok(())
}


use wasm_bindgen_test::wasm_bindgen_test;
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
use web_sys::console;

#[wasm_bindgen_test]
fn test_wasm_convert() {
    let req = RequestData {
        text: "hello world".to_string(),
        operation: ClientRequestType::MapRes,
    };

    let js_value = serde_wasm_bindgen::to_value(&req).unwrap();
    let res = ConvertMain(js_value).unwrap();

    let output: ResponseData = serde_wasm_bindgen::from_value(res).unwrap();
    match output.phonetic {
        PhoneticResponceType::MapRes(pairs) => {
            assert!(!pairs.is_empty());
            println!("{:?}", pairs);
            
            console::log_1(&format!("{:?}", pairs).into());
        } ,
        _ => panic!("Expected Map response"),
    }
}
