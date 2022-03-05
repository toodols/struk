use crate::deserialize;
use nom::error::ErrorKind;
use serde_json::{Value, json};
use std::str;

use crate::language::{Primitive, ParserValue, ArrayLength, parse};

pub fn serialize<'a>(rule: &ParserValue, input: &'a Value) -> Result<Vec<u8>, &'a str> {
	let b = to_bytes(rule, input).unwrap();
	Ok(b)
}

fn to_bytes<'a>(rule: &ParserValue, input: &'a Value) -> Result<Vec<u8>, &'a str> {
	match rule {
		ParserValue::String(len)=>{
			match len {
				ArrayLength::Fixed(fixedLen)=>{
					let s = input.as_str().ok_or("not string sad")?;
					if s.len()==*fixedLen {
						Ok(s.as_bytes().to_vec())
					} else {
						Err("string does not match fixed length")
					}
				}
				ArrayLength::Bit(_) => todo!(),
				ArrayLength::Null => todo!(),
			}
		}
		ParserValue::Array(val, b)=>{
			let e = input.as_array().ok_or("not array sad")?;
			
			// val.as_ref()
			todo!()
		}
		ParserValue::Primitive(primitive) => {
			match primitive {
				Primitive::U64 => todo!(),
				Primitive::I64 => todo!(),
				Primitive::F64 => todo!(),
				Primitive::U32 => {
					let val = input.as_u64().ok_or("not number")?;
					if val <= 4294967295 {
						Ok((val as u32).to_be_bytes().to_vec())
					} else {
						Err("too big for u32")
					}
				},
				Primitive::I32 => todo!(),
				Primitive::F32 => todo!(),
				Primitive::U16 => {
					let val = input.as_u64().ok_or("not number")?;
					if val <= 65535 {
						Ok((val as u16).to_be_bytes().to_vec())
					} else {
						Err("too big for u16")
					}
				},
				Primitive::I16 => todo!(),
				Primitive::U8 => {
					let val = input.as_u64().ok_or("not number")?;
					if val <= 255 {
						Ok(vec!(val as u8))
					} else {
						Err("too big for u8")
					}
				},
				Primitive::I8 => todo!(),
				Primitive::Bool => todo!(),
			}
		},
		ParserValue::Tuple(rules) => {
			let values = input.as_array().ok_or("tuple: not array sad")?;
			let mut val: Vec<u8> = Vec::new();
			for i in 0..rules.len() {
				let mut a = to_bytes(&rules[i], &values[i])?;
				val.append(&mut a);
			};
			Ok(val)
		},
		ParserValue::Enum(values) => {
			let mut val: Result<Vec<u8>, &str> = Err("no values apply to this enum");
			for rule_num in 0..values.len() {
				let rule = &values[rule_num];
				let result = to_bytes(rule, input);
				if result.is_ok() {
					val = result;
					break;
				}
			}
			return val;
		},
		ParserValue::Struct(pairs) => {
			let values = input.as_object().ok_or("not object sad")?;
			let mut val: Vec<u8> = Vec::new();
			for (k, v) in pairs {
				val.append(&mut to_bytes(v.as_ref(), values.get(k).ok_or("key not there")?)?);
			};
			Ok(val)
		},
		ParserValue::Map(_, _) => todo!(),
		ParserValue::BoolStruct(_) => todo!(),
		ParserValue::Literal(_) => Ok(Vec::new()),
	}	
}

#[test]
fn test(){
	// make a rule, {x: u32, y: u32, z: u32} is an (integer) vector3. integers because floats aren't implemented
	let my_rule= &parse::<(&str, ErrorKind)>(
		"{x: u32, y: u32, z: u32}"
	).unwrap().1;
	
	// this is what is being parsed. a value that fits the rule
	let jsonval = json!({"x": 0, "y": 1000, "z": 100});

	// we serialize it into bytes here. since u32 is 4 bytes, and there are 3 u32's, the result will be 12 bytes.
	let bytes = serialize(my_rule, &jsonval).unwrap();

	println!("Bytes: {:?}", bytes);

	// the original value can be retrieved by deserializing it again. it will be identical to jsonval.
	let a = serde_json::to_string(&deserialize(my_rule, bytes).unwrap()).unwrap();
	println!("Deserialized: {}", a)
}