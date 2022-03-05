use byteorder::{BigEndian, ReadBytesExt};
use serde_json::{json, Map};
use serde_json::value::Value;
use std::io::Cursor;
use std::{io::Read, str};

use crate::language::{ParserValue, Primitive, Literal, ArrayLength};

fn getCountFromBits(cursor: &mut Cursor<Vec<u8>>, num: u8) -> Result<usize, std::io::Error> {
    let count: std::io::Result<u32> = match num {
        1=>{cursor.read_u8().map(|e| e as u32)}
        2=>{cursor.read_u16::<BigEndian>().map(|e| e as u32)}
        3=>{cursor.read_u24::<BigEndian>()}
        4=>{cursor.read_u32::<BigEndian>()}
        _=>{panic!("bad")}
    };
    Ok(count? as usize)
}

fn parse<'a>(rule: &ParserValue, cursor: &mut Cursor<Vec<u8>>) -> Result<Value, std::io::Error> {
    match rule {
        ParserValue::Primitive(Primitive::U32) => {
            Ok(json!(cursor.read_u32::<BigEndian>()?))
        }
        ParserValue::Primitive(Primitive::U16) => {
            Ok(json!(cursor.read_u16::<BigEndian>()?))
        }
        ParserValue::Primitive(Primitive::U8) => {
            Ok(json!(cursor.read_u8()?))
        }
        ParserValue::Primitive(Primitive::Bool) => {
            Ok(json!(if cursor.read_u8()?>0 {true} else {false}))
        }
        ParserValue::Array(value, len_vec) => {
            let mut nums: Vec<usize> =  Vec::with_capacity(len_vec.len());

            for len in len_vec {
                match len {
                    ArrayLength::Fixed(num) => {
                        nums.push(*num)
                    }
                    ArrayLength::Bit(num) => {
                        nums.push(getCountFromBits(cursor, *num)?)
                    }
                    ArrayLength::Null => {
                        panic!("Null array length bad");
                    }
                }
            };
            
            // i had no choice but to settle for recursion https://stackoverflow.com/questions/9555864/variable-nested-for-loops stack overflow answer seemed too complicated
            fn inner(cursor: &mut Cursor<Vec<u8>>, value: &ParserValue, nums: &Vec<usize>, depth: usize) -> Result<Vec<Value>, std::io::Error> {
                let mut arr: Vec<Value> = Vec::with_capacity(nums[depth] as usize);
                for _ in 0..nums[depth] {
                    if depth>=nums.len()-1 {
                        arr.push(parse(value, cursor)?);
                    } else {
                        arr.push(json!(inner(cursor, value, nums, depth+1)?))
                    }
                }
                return Ok(arr);
            };
            

            Ok(json!(inner(cursor, value.as_ref(), &nums, 0)?))
        }
        ParserValue::Literal(n)=>{
            Ok(match n {
                Literal::Null => {
                    json!(null)
                }
                Literal::Int(n) => {
                    json!(n)
                }
                Literal::Bool(b)=>{
                    json!(b)
                }
                Literal::String(s)=>{
                    json!(s)
                }
            })
        }
        ParserValue::Struct(map) => {
            let mut a = Map::new();
            for (key, rule) in map.iter() {
                a.insert(key.clone(), parse(rule.as_ref(), cursor)?);
            }
            Ok(json!(a))
        }
        ParserValue::Map(key, value)=> {
            let len = cursor.read_u16::<BigEndian>()?;
            let mut map = Map::new();
            for _ in 0..len {
                map.insert(parse(key, cursor)?.to_string(),json!(parse(value, cursor)?));
            };
            Ok(json!(map))
        }
        ParserValue::String(len) => match len {
            ArrayLength::Fixed(num) => {
                let mut buf = vec![0u8; *num as usize];
                cursor.read_exact(&mut buf)?;
                Ok(json!(str::from_utf8(&buf).unwrap()))
            }
            ArrayLength::Null => {
                let mut text: Vec<u8> = Vec::new();
                loop {
                    let mut buf = [0; 1];
                    cursor.read_exact(&mut buf)?;
                    if buf[0] == 0 {
                        break
                    }
                    text.push(buf[0]);
                };
                Ok(json!(str::from_utf8(&text).unwrap()))
            }
            ArrayLength::Bit(num)=>{
                let count = getCountFromBits(cursor, *num)?;
                let mut buf = vec![0u8; count];
                cursor.read_exact(&mut buf)?;
                Ok(json!(str::from_utf8(&buf).unwrap()))
            }
        },
        ParserValue::Enum(v)=>{
            let rule = &v[cursor.read_u8()? as usize];
            parse(rule, cursor)
        }
        ParserValue::Tuple(v) => v.iter().map(|elem| parse(elem, cursor)).collect(),
        _ => panic!("sad"),
    }
}

pub fn deserialize<'a>(rule: &ParserValue, input: Vec<u8>) -> Result<Value, &'static str> {
    let mut cursor = Cursor::new(input);
    parse(rule, &mut cursor).map_err(|_|"There is a 99% chance that the buffer is not long enough")
}