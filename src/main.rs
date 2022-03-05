// still learning rust be gentle
use std::fs;
use std::fs::write;
use std::str;

mod language;
use language::parse;

mod deserialize;
use deserialize::{deserialize};
use nom::error::ErrorKind;

mod serialize;
use serde_json::json;
use serialize::{serialize};

fn main() {
    let rule_text = &fs::read_to_string("./photoprequest.struk").expect("oh no")[..];

    // let rule_text = "{r: u8, g: u8, b: u8, name: \"Color\"}";
    let (_rest, rule) = parse::<(&str,ErrorKind)>(
        rule_text
    ).expect("Failed to parse language");

    println!("RULE: {}", rule_text);
    println!("PARSED RULE: {:?}", rule);

    let d = &serialize(&rule, &json!({})).unwrap()[..];
    
    
    let data = [0,1,0,1,65, 0,0, 0,1, 1, 0,0, 0,0, 0,0,0,1, 0,0, 1, 0,0,0,1, 0,0];
    // let data = [
    //     &[255,0,0],
    //     "Name".as_bytes(),
    //     &[0]
    // ].concat();
    let result = deserialize(&rule, data.to_vec()).unwrap();

    write("data.dat", data).unwrap();
    println!("BINARY ARRAY: {:?}", data);
    println!("RESULT: {}", result);
}
