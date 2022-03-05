use nom::bytes::complete::is_a;
use std::vec::IntoIter;
use std::{fmt, iter::Map, str};

use nom::character::complete::digit1;
use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while},
    character::complete::{alphanumeric1 as alphanumeric, char, one_of},
    combinator::{cut, map, opt, value},
    error::{context, convert_error, ContextError, ErrorKind, ParseError, VerboseError},
    multi::{many0, separated_list0, separated_list1},
    number::complete::double,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    Err, IResult,
};

#[derive(Clone, Debug)]
pub enum ArrayLength {
    Bit(u8),      // 1b, 2b, 3b, 4b, ...
    Null,         // null terminated
    Fixed(usize), //0
}

#[derive(Clone, Debug)]
pub enum Primitive {
    U64,
    I64,
    F64,
    U32,
    I32,
    F32,
    U16,
    I16,
    U8,
    I8,

    Bool,
}

#[derive(Clone, Debug)]
pub enum Literal {
    Int(u32),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Clone, Debug)]
pub enum ParserValue {
    String(ArrayLength), // str
    Array(Box<ParserValue>, Vec<ArrayLength>), // u8[]
    Tuple(Vec<ParserValue>), // (str, u8)
    Enum(Vec<ParserValue>), // str | u8
    Struct(Vec<(String, Box<ParserValue>)>), // {a: str}
    Map(Box<ParserValue>, Box<ParserValue>), // {[str]: u32}
    BoolStruct(Vec<String>), // bitfield type {a,b,c,d,e,f}
    Primitive(Primitive), // bool, u32
    Literal(Literal) // "abcdef", 1234, true, false
}

fn sp<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";

    // nom combinators like `take_while` return a function. That function is the
    // parser,to which we can pass the input
    take_while(move |c| chars.contains(c))(i)
}

fn identifier<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E>{
    is_a("1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_$")(input)
}
fn parse_string_literal<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Literal, E> {
    map(preceded(char('"'), terminated(identifier, char('"'))), |e: &str|Literal::String(String::from(e)))(input)
}

fn parse_literal<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, ParserValue, E> {
    map(alt((map(tag("null"), |_|Literal::Null),map(tag("true"), |_|Literal::Bool(true)),map(tag("false"), |_|Literal::Bool(false)),map(digit1, |e:&str|Literal::Int(e.parse::<u32>().unwrap())), parse_string_literal)), |e|ParserValue::Literal(e))(input)
}
fn parse_primitive<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, ParserValue, E> {
    map(
        alt((
            value(Primitive::U64, tag("u64")),
            value(Primitive::U32, tag("u32")),
            value(Primitive::U16, tag("u16")),
            value(Primitive::U8, tag("u8")),
            value(Primitive::Bool, tag("bool")),
        )),
        |a| ParserValue::Primitive(a),
    )(input)
}

fn parse_tuple<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, ParserValue, E> {
    map(
        preceded(
            tuple((char('('), sp)),
            cut(terminated(
                separated_list1(preceded(sp, terminated(char(','), sp)), parse_value),
                preceded(sp, char(')')),
            )),
        ),
        |v| ParserValue::Tuple(v),
    )(input)
}

fn parse_enum<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, ParserValue, E> {
    map(
        terminated(
            separated_list1(preceded(sp, terminated(char('|'), sp)), parse_value),
            sp,
        ),
        |v| {
            // im too lazy to make a good parser
            if v.len() == 1 {
                return v[0].clone();
            }
            ParserValue::Enum(v)
        },
    )(input)
}

fn parse_array_suffix<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Vec<ArrayLength>, E> {
    map(
        alt((
            map(tag("[]"), |s| ArrayLength::Bit(2)),
            map(
                preceded(
                    tuple((char('['), sp)),
                    terminated(digit1, tuple((sp, char(']')))),
                ),
                |s: &str| {
                    ArrayLength::Fixed(s.parse::<usize>().unwrap())
                },
            ),
        )),
        |a| vec![a],
    )(input)
}


// str / (char "[" sp ("null" / digit1 / ([1234] "b")) sp "]")
fn parse_string_type<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, ParserValue, E> {
    alt((map(tag("str"), |_|ParserValue::String(ArrayLength::Bit(2))), map(preceded(tuple((tag("char"), char('['), sp)), terminated(alt((
        map(tag("null"), |_|ArrayLength::Null),
        map(digit1, |n: &str|ArrayLength::Fixed(n.parse::<usize>().unwrap())),
        map(terminated(is_a("1234"), char('b')), |n: &str|ArrayLength::Fixed(n.parse::<usize>().unwrap()))
    )),tuple((sp, char(']'))))), |a|ParserValue::String(a))))(input)
}

// "(" value_and_enum ")" 
fn parse_parentheses<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, ParserValue, E> {
    preceded(char('('), terminated(parse_value_and_enum, char(')')))(input)
}

// parentheses / struct / primitive / tuple array_suffix*
// level 0 (inside)
fn parse_value<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, ParserValue, E> {
    map(
        tuple((
            alt((parse_parentheses, parse_struct, parse_string_type, parse_primitive, parse_literal, parse_tuple)),
            many0(parse_array_suffix),
        )),
        |(val, arr)| {
            let mut ret = val;
            for v in arr.iter() {
                ret = ParserValue::Array(Box::new(ret), v.to_vec())
            }
            ret
        },
    )(input)
}

// "{" (identifier sp ":" sp value_and_enum )* "}"
fn parse_struct<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, ParserValue, E> {
    map(preceded(
        tuple((char('{'), sp)),
        terminated(separated_list0(tuple((sp, char(','), sp)), tuple((terminated(identifier, tuple((sp, char(':'), sp))), parse_value_and_enum))), tuple((sp, char('}')))),
    ), |e|{
        let b = e.iter().map(|(e,a )|(String::from(*e), Box::from(a.clone()))).collect();
        ParserValue::Struct(b)
    })(input)
}

fn parse_value_and_enum<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, ParserValue, E> {
    preceded(sp, terminated(alt((parse_enum, parse_value)), sp))(input)
}

#[test]
fn test_everything() {
    let rule_text = "(
        u8,
        char[10],
        {r: u8, g: u8, b: u8},
        {prop: (u8 | str | {nested: bool})}[]
    ) | u8";
    println!("{}", rule_text);
    let e = parse::<(&str, ErrorKind)>(rule_text);
    println!("{:?}", e)
}

fn validate(){
    todo!()    
}

pub fn parse<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, ParserValue, E> {
    parse_value_and_enum(input)
}
