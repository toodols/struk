# Struk
Simple language for data serialization with heavy inspiration from Typescript, and a bit of Rust (number names) and C# (Multi dimensional arrays)

Right now not done.


## Types

### Numbers
|          | 1 byte (2^8 poss.)       | 2 byte (2^16 poss.)              | 4 byte (2^32 poss.)                                                                                                     | 8 byte (2^64 poss.)                                                                                                      |
|----------|--------------------------|----------------------------------|-------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------|
| signed   | `i8` - *char* `-128~127` | `i16` - short `-32768~32767`     | `i32` - int `-2147483648~2147483647`                                                                                    | `i64` - long                                                                                                             |
| unsigned | `u8` - *char* `0~255`    | `u16` - unsigned short `0~65536` | `u32` - unsigned int `4294967296`                                                                                       | `u64` - unsigned long                                                                                                    |
| float    |                          |                                  | `f32` - [float](https://docs.microsoft.com/en-us/cpp/c-language/type-float?view=msvc-170#range-of-floating-point-types) | `f64` - [double](https://docs.microsoft.com/en-us/cpp/c-language/type-float?view=msvc-170#range-of-floating-point-types) |

#### **Example**
Syntax: `u16`
|Input|Value|
|-----|-----|
| `00 0F` | `15` |
| `00 00` | `0` |
| `00 FF` | `255` |
| `03 E8` | `1000` |

### Boolean
`bool` - 1 byte, unless exclusively inside a struct or an array.

### Array
Add [] to the end of.
`bool[]`

### Struct
`{r: i32, g: i32, b: i32}`

If the struct is exclusively booleans, it will automatically be interpreted as a bitfield.
* Ex: `{a: bool, b: bool, c: bool, e: bool, f: bool}`
* Ex: `{a,b,c,d,e,f}` This is also valid

### Dictionary
`{[str]: i32}`
`{[u32]: bool}`

*Yes you can have fixed length keys!*
`{[char[4]]: str}`

### Variable length strings
- `str` - Alias of `char[2b]`

- `char[1b]` - String with a max length of 255
- `char[2b]` - String with a max length of 65535
- `char[3b]` - String with a max length of 16777215
- `char[4b]` - String with a max length of 4294967295

- `char[null]` - Null terminated string. No length limits provided there is no null character inside the string.

### Fixed length strings
- `char[10]` - String with length of 10. (10 bytes)
- `char[20]` - String with length of 20. (20 bytes)

### Tuples
Wrap stuff in parentheses and commas in between.
`(str, u32)`

*Example*
| `(str, u32)` | `00 05` | `41 70 70 6C 65` | `00 00 00 04` |
|-|-|-|-|
| `["Apple", 4]` | Length of 5 | "Apple" | 4 |

### Enum
`("Red" | "Green" | "Blue")`

Example: 
``

(str | null)

### Predefined values
This can eliminate the need for post processing but it should be used for tuples.

## Wait what about ProtoBuf?
*shhhhhhhh*