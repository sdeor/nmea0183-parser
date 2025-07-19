# A Rust procedural macro for NMEA 0183-style content parsing

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../LICENSE-MIT)
[![Apache License 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](../LICENSE-APACHE)
[![docs.rs](https://docs.rs/nmea0183-derive/badge.svg)](https://docs.rs/nmea0183-derive)
[![Crates.io Version](https://img.shields.io/crates/v/nmea0183-derive.svg)](https://crates.io/crates/nmea0183-derive)

Based on [`nom-derive`] and with a lot of similarities, `nmea0183-derive` is a custom
derive attribute to derive content parsers for your NMEA 0183-style data structures.

It is not meant to replace [`nmea0183-parser`], but to work alongside it, providing
a convenient and easy way to derive parsers for your data structures without having
to write boilerplate code.

[`nmea0183-parser`]: https://crates.io/crates/nmea0183-parser
[`nom-derive`]: https://crates.io/crates/nom-derive
[`nom`]: https://crates.io/crates/nom

## `#[derive(NmeaParse)]`

The `NmeaParse` derive macro automatically generates an implementation of the `NmeaParse` trait for your structs and enums using [`nom`] parsers when possible. This allows you to define your data structures and derive parsing logic without writing boilerplate code.

You can use the `#[derive(NmeaParse)]` attribute on your structs and enums to automatically generate parsing logic based on the field types. The macro will try to infer parsers for known types (implementors of the `NmeaParse` trait), but you can also customize the parsing behavior using attributes.

## Basic usage

Import the `NmeaParse` derive macro and use it on your structs or enums:

```rust
use nmea0183_parser::NmeaParse;

#[derive(NmeaParse)]
struct Data {
    pub id: u8,
    pub value: f64,
    pub timestamp: u64,
}
```

This also works with:

- Tuple structs

  ```rust
  use nmea0183_parser::NmeaParse;

  #[derive(NmeaParse)]
  struct Data(u8, f64, u64);
  ```

- Unit structs

  ```rust
  use nmea0183_parser::NmeaParse;

  #[derive(NmeaParse)]
  struct Data;
  ```

- And enums

  ```rust
  use nmea0183_parser::NmeaParse;

  #[derive(NmeaParse)]
  #[nmea(selector(u8::parse))]
  enum Data {
      #[nmea(selector(0))]
      TypeA,
      #[nmea(selector(1))]
      TypeB(f64),
      #[nmea(selector(2))]
      TypeC { values: Vec<u8> },
  }
  ```

## Default parsing function

By default, the `NmeaParse` derive macro will use the `T::parse` function to parse the fields of your struct or enum, where `T` is the type of the field.

This function can automatically derive by the `NmeaParse` derive or by implementing the `NmeaParse` trait manually.

For example (using `NmeaParse` derive):

```rust
use nmea0183_parser::NmeaParse;

#[derive(NmeaParse)]
struct S {
    a: u8,
}

#[derive(NmeaParse)]
struct S2 {
    b: u8,
    s: S,
}
```

Or manually implementing the `NmeaParse` trait:

```rust
use nmea0183_parser::{IResult, NmeaParse};
use nom::{AsChar, Input, error::ParseError};

// No NmeaParse derive
struct S {
    a: u8,
}

#[derive(NmeaParse)]
struct S2 {
    b: u8,
    s: S,
}

impl<I, E> NmeaParse<I, E> for S
where
    I: Input,
    <I as Input>::Item: AsChar,
    E: ParseError<I>,
{
    fn parse(i: I) -> IResult<I, Self, E> {
        let (i, a) = nom::character::complete::u8(i)?;
        Ok((i, S { a }))
    }
}
```

## Attributes

Derived parsers can be customized using `nmea` attribute annotation with sub-attributes. These attributes allow you to specify how the parser should behave for specific fields or variants. For example `#[nmea(parse_as(u8))]`.

You can specify attribute arguments using either literal strings, such as `#[nmea(parse_as = "u8")]`, or parenthesized values, such as `#[nmea(parse_as(u8))]`.

> **Note:** Order of attributes is important! `#[nmea(cond(true), map(|x| x + 1))]` is not the same as `#[nmea(map(|x| x + 1), cond(true))]`, as the former will map the value and then applies the condition (wrapping the value in `Some` in this case), while the latter will apply the condition first, generating a compile error in the map since there is no `Add` implementation for `{integer}` and `Option<T>`.

The following attributes are supported:

| Attribute                                           | Level     | Description                                                                                         |
| --------------------------------------------------- | --------- | --------------------------------------------------------------------------------------------------- |
| [cond](#conditional-parsing)                        | field     | Specifies a condition for when the field should be parsed, return an `Option<T>`                    |
| [exact](#exact-parsing)                             | top-level | Ensures that the input is fully consumed by the parser                                              |
| [ignore](#ignore-fields)                            | field     | Ignores the field during parsing and sets its value to `Default::default()`                         |
| [into](#into-conversion)                            | field     | Automatically converts the parsed result to another type                                            |
| [map](#mapping-parsed-values)                       | field     | Maps the parsed value to another type                                                               |
| [parse_as](#custom-parsing-types)                   | field     | Specifies the type to use when parsing the field                                                    |
| [parser](#custom-parsers)                           | field     | Specifies a custom parser function for the field                                                    |
| [pre_exec](#pre-execution-and-post-execution-code)  | both      | Executes Rust code before parsing a field or structure                                              |
| [post_exec](#pre-execution-and-post-execution-code) | both      | Executes Rust code after parsing a field or structure                                               |
| [selector](#selector-and-selection-error)           | both      | Specifies the value used to match an enum variant                                                   |
| [selection_error](#selector-and-selection-error)    | top-level | Specifies the error to return if the selector fails to match                                        |
| [separator](#custom-separator)                      | none      | Intended to specify the separator between fields (currently not supported, defaults to `char(',')`) |
| [skip_after](#skip-before-and-after-parsing)        | both      | Skips a specified number of characters after parsing a field or structure                           |
| [skip_before](#skip-before-and-after-parsing)       | both      | Skips a specified number of characters before parsing a field or structure                          |

Except for `cond`, `map`, `pre_exec`, and `post_exec`, top-level attributes can only appear once per struct or enum, and field attributes can only appear once per field or variant.

### Custom parsers

When the default parsing function is not what you want or does not provided for your type, you can specify a custom parser using the `parser(parser_function)` attribute. The `parser_function` should be a function that takes an input of type `I` and returns an `IResult<I, T, E>`, where `T` is the type of the field being parsed.

For example:

```rust
use nmea0183_parser::{IResult, NmeaParse};
use nom::error::ParseError;

#[derive(NmeaParse)]
struct Data {
    #[nmea(parser(parse_custom))]
    value: f64,
}

fn parse_custom<'a, E>(input: &'a str) -> IResult<&'a str, f64, E>
where
    E: ParseError<&'a str>,
{
    Ok((input, 42.0)) // Example parsing logic
}
```

> Note: While your custom parser function does not need to use a generic input type (`I`), it must use a generic error type `E`. This ensures compatibility with the derive macro's parsing infrastructure.

The `parser` argument can be a complex expression:

```rust
use nmea0183_parser::NmeaParse;
use nom::{combinator::map, number::complete::double};

#[derive(NmeaParse)]
struct Data {
    count: u8,
    #[nmea(parser(map(double, |v| v * 2.0)))]
    value: f64,
}
```

### Custom parsing types

You can specify a type to parse as using the `#[nmea(parse_as(type))]` attribute, which will use the specified type's parsing function instead of the default one. This is useful when you want to parse a field as a specific type that implements `NmeaParse`. For simple conversions this approach is preferred over using a custom parser, since it allows the derive macro to use `U::parse_preceded` when needed.

```rust
#[derive(NmeaParse)]
struct Data {
    count: u8,
    #[nmea(map(|v: u32| v as f64), parse_as(u32))]
    value: f64, // Used `u32::parse_preceded` to parse the value, then mapped to `f64`
}
```

### Ignore fields

If a field is marked with `ignore`, it will not be parsed from the input, and its value will be set to the default value for its type. The field's type must implement `Default`, otherwise a compile error will occur.

Special care is taken when ignoring the first field of a struct: the macro will use the regular `parse` function for the next field, instead of `parse_preceded`, to ensure correct parsing alignment.

If the struct is nested within another struct, the "first" field may not actually be first in the overall input. To address this, the top level struct will consume the separator before parsing the nested struct if needed, allowing the nested struct to be parsed as if it were the first field in the remaining input.

```rust
#[derive(NmeaParse)]
struct Data {
    #[nmea(ignore)]
    a: u8,      // Ignored, set to Default::default()
    b: u16,     // parsed with `parse_preceded`, but now parsed with `parse` since the first field is ignored
    c: i32,
}

#[derive(NmeaParse)]
struct Data2 {
    d: u8,
    e: Data,    // `Data` is parsed with `parse_preceded`, so the separator is consumed
                // and `e` is treated as the first field of the remaining input
}
```

### Conditional parsing

The `cond` attribute is used to conditionally parse a field. If the condition evaluates to `true`, the field is parsed and wrapped in `Some`, otherwise it is set to `None`. The condition can be any expression that evaluates to a boolean value.

The conditional parsing applies to the whole field being present - both the separator and the value. If the condition is not met, the parser will not consume the separator. This is used when the field may or may not be present in the input data at all, i.e. either "<previous_field>,<current_field>" or "<previous_field>"; notice the lack of comma in the latter case.

```rust
#[derive(NmeaParse)]
struct Data {
    a: u8,
    #[nmea(cond(a > 0))]
    b: Option<f64>, // If `a` is greater than 0, the parser will use `f64::parse_preceded` and set `b` to `Some(value)`,
                    // otherwise it will set `b` to `None`
    c: u8,
}

let result = Data::parse("0,1");    // `a` is 0, so `b` sets to `None`.
                                    // The separator is not consumed yet, allowing `c` to be parsed correctly as `1`.
let result = Data::parse("0,,2");   // `a` is 0, so `b` sets to `None`. When the parser reaches `c` it tries to parse
                                    // the next field, which is empty, leading to parse error.
let result = Data::parse("1,2,3");  // `a` is 1, so `b` sets to `Some(2.0)`, and `c` is parsed as `3`.
```

This is different approach than the following example, which uses `#[nmea(parser(cond(condition, parser_function)))]` to conditionally apply a parser function:

```rust
#[derive(NmeaParse)]
struct Data {
    a: u8,
    #[nmea(parser(cond(a > 0, f64::parse)))]
    b: Option<f64>, // The parser will use `nom::sequence::preceded(<separator>, cond(a > 0, f64::parse))`.
                    // If `a` is greater than 0, the parser will parse the next field as `f64` and set `b` to `Some(value)`,
                    // otherwise it will set `b` to `None`. Either way, the separator is consumed.
    c: u8,
}

let result = Data::parse("0,1");    // `a` is 0, so `b` sets to `None`. The separator is consumed by `b`,
                                    // leading to parse error for `c` since expected a separator but found `1`.
let result = Data::parse("0,,2");   // `a` is 0, so `b` sets to `None`. The separator is consumed by `b`,
                                    // allowing `c` to be parsed correctly as `2`.
let result = Data::parse("1,2,3");  // `a` is 1, so `b` sets to `Some(2.0)`, and `c` is parsed as `3`.
```

In this case, even if the condition is not met, the parser will still consume the separator. This is used when the field is always present in the input data but might be empty, i.e. either "<previous_field>,<current_field>,<next_field>" or "<previous_field>,,<next_field>"; notice the empty field in the latter case.

### Mapping parsed values

The `map` attribute allows you to apply a function to the parsed value before it is returned. It is often combined with the `parse` or `parse_as` attributes to transform the parsed value into a different type or format.

```rust
#[derive(NmeaParse)]
struct Data {
    #[nmea(map(|v: u32| v.to_string()), parse_as(u32))]
    a: String,
}
```

### Into conversion

The `into` attribute automatically converts the parsed output types into other types.
It requires the output types to implement the `Into` trait.

```rust
fn parser<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    Ok(("", input))
}

#[derive(NmeaParse)]
struct Data {
    #[nmea(into, parser(parser))]
    a: Vec<u8>, // The parser returns a `&str`, which is then converted into a `Vec<u8>`.
}
```

### Exact parsing

The `exact` attribute is a top-level attribute ensures that the input is fully consumed by the parser. If there are any remaining characters in the input after parsing, an error will be returned.

```rust
#[derive(NmeaParse)]
#[nmea(exact)]
struct Data {
    a: u8,
}

let result = Data::parse("1"); // Ok(("", Data { a: 1 }))
let result = Data::parse("1,2"); // Err(Error { input: ",2", code: Verify })) - input not fully consumed
```

### Pre-execution and post-execution code

The `pre_exec` and `post_exec` attributes execute Rust code before and after parsing a field or structure.

Those attributes can be specified multiple times, and the code will be executed in order.

The current input is available as a variable named `nmea_input`. If a variable with the same name is created it will be used as the input, resulting in side effects.

```rust
#[derive(NmeaParse)]
#[nmea(post_exec(dbg!(nmea_input);))]
struct Data {
    #[nmea(pre_exec(let length = nmea_input.len();))]
    a: u8,
    #[nmea(parser(|i| Ok((i, length))))]
    size: usize,
}
```

### Skip before and after parsing

The `skip_before` and `skip_after` attributes allow you to skip a specified number of bytes before or after parsing a field or structure. This is useful when you want to ignore certain characters in the input that are not part of the data you want to parse.

```rust
#[derive(NmeaParse)]
struct Data {
    #[nmea(skip_before(1))]
    a: u8,
    #[nmea(skip_after(1))]
    b: u8,
}
```

### Selector and selection error

When parsing enums, the `selector` attribute must be used to specify the value that will be used to match an enum variant. It must be applied to the enum and its variants.

- At the structure level, it specifies a parser function that will be used to parse the selector value.
- At the variant level, it specifies the value that will be used to match the variant. Note this expression can contain a pattern guard, such as `value if value > 0`.

```rust
#[derive(NmeaParse)]
#[nmea(selector(i8::parse))]
enum Data {
    #[nmea(selector(0))]
    TypeA,
    #[nmea(selector(1 | 2))]
    TypeB(f64),
    #[nmea(selector(value if value > 0))]
    TypeC { values: Vec<u8> },
}
```

The generated parser of this enum will first parse the selector value using the specified parser function, then consume the separator, and finally match the parsed value against the variant selectors.

By default, if no variant matches the parsed selector value, a `nom` error with `ErrorKind::Switch` is raised.
You can use `_` as a catch-all variant to handle unmatched selectors. This variant must be defined last in the enum or a compile error will occur.

```rust
#[derive(NmeaParse)]
#[nmea(selector(i8::parse))]
enum Data {
    #[nmea(selector(value if value > 0))]
    TypeA,
    #[nmea(selector(_))]
    TypeB(f64),
}
```

If you want to specify a custom error to return when the selector fails to match, you can use the `selection_error` attribute at the top-level of the enum and provide a custom error. The error must be `nmea0183_parser::Error<I, E>`.

```rust
use nmea0183_parser::{Error};

#[derive(NmeaParse)]
#[nmea(selection_error(Error::Unknown))]
enum Data {
    #[nmea(selector(0))]
    TypeA,
    #[nmea(selector(1 | 2))]
    TypeB(f64),
    #[nmea(selector(value if value > 0))]
    TypeC { values: Vec<u8> },
}
```

### Custom separator

The `separator` attribute is intended to specify the separator between fields. However, it is currently not supported and defaults to `char(',')`. This means that the parser will expect fields to be separated by commas.

## Generic Type Parameters

The `NmeaParse` derive macro fully supports generic type parameters on structs and enums. When you use generics, the macro automatically adds the necessary trait bounds (such as `T: NmeaParse`) to ensure that parsing works seamlessly for any type that implements the `NmeaParse` trait.

For example:

```rust
use nmea0183_parser::NmeaParse;

#[derive(NmeaParse)]
struct Data<T> {
    a: T,
}

let result: IResult<_, Data<u32>> = Data::parse("1234");
assert!(matches!(result, Ok(("", Data { a: 1234 }))));
```
