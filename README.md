# A Flexible NMEA Framing Parser for Rust

This Rust library provides a generic and configurable parser for NMEA 0183-style messages, with the typical format:

```compile_fail
$HHH,D1,D2,...,Dn*CC\r\n
```

It focuses on **parsing and validating the framing** of NMEA-like sentences (start character, comma-separated fields, optional checksum, and optional CRLF), allowing you to plug in your own domain-specific content parsers.

---

## ✨ Why Use This Library?

Unlike traditional NMEA libraries that tightly couple format and content parsing, `nmea0183_parser` lets you:

- ✅ **Choose your compliance level** (strict vs lenient)
- ✅ **Plug in your own payload parser** (GNSS, marine, custom protocols)
- ✅ **Support both `&str` and `&[u8]` inputs**
- ✅ **Get detailed error types for robust debugging**
- ✅ **Parse without allocations**, built on top of [`nom`](https://github.com/Geal/nom)

Perfect for:

- GNSS/GPS receiver integration
- Marine electronics parsing
- IoT devices consuming NMEA-like protocols
- Debugging or testing tools for embedded equipment
- Legacy formats that resemble NMEA but don’t strictly comply

---

## 📦 Features

- ✅ ASCII-only validation
- ✅ Required or optional checksum validation
- ✅ Required or forbidden CRLF ending enforcement
- ✅ Zero-allocation parsing
- ✅ Built on `nom` combinators
- ✅ Fully pluggable content parser (you bring the domain logic)

---

## 🚀 Getting Started

Add the library to your project:

```toml
[dependencies]
nmea0183_parser = { git = "https://github.com/sdeor/nmea0183_parser" }
```

---

### ✏️ Basic Example

```rust
use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, nmea0183};
use nom::Parser;

// Your custom logic for the inner data portion (after "$XXX," and before "*CC")
fn parse_content(i: &str) -> IResult<&str, bool> {
    // You can decode fields here, split by commas, etc.
    Ok((i, true))
}

fn main() {
    // Create a parser factory with required checksum and CRLF
    let parser_factory = nmea0183(ChecksumMode::Required, LineEndingMode::Required);
    let mut parser = parser_factory(parse_content);

    // Try parsing a message
    let result = parser.parse("$GPGGA,123456,data*41\r\n");
    assert!(result.is_ok());
}
```

---

### 🔧 Configuration Options

This parser separates **framing logic** from your **content parsing**, letting you choose how strict to be.

```rust
use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, nmea0183};
use nom::Parser;

fn content_parser(i: &str) -> IResult<&str, bool> {
    Ok((i, true))
}

// Strict: checksum and CRLF both required
let mut strict = nmea0183(ChecksumMode::Required, LineEndingMode::Required)(content_parser);
assert!(strict.parse("$GPGGA,data*6A\r\n").is_ok());
assert!(strict.parse("$GPGGA,data*6A").is_err());  // Missing CRLF
assert!(strict.parse("$GPGGA,data\r\n").is_err()); // Missing checksum

// Checksum required, CRLF forbidden
let mut no_crlf = nmea0183(ChecksumMode::Required, LineEndingMode::Forbidden)(content_parser);
assert!(no_crlf.parse("$GPGGA,data*6A").is_ok());
assert!(no_crlf.parse("$GPGGA,data*6A\r\n").is_err());
assert!(no_crlf.parse("$GPGGA,data").is_err());

// Checksum optional, CRLF required
let mut optional_checksum = nmea0183(ChecksumMode::Optional, LineEndingMode::Required)(content_parser);
assert!(optional_checksum.parse("$GPGGA,data*6A\r\n").is_ok());
assert!(optional_checksum.parse("$GPGGA,data\r\n").is_ok());
assert!(optional_checksum.parse("$GPGGA,data*99\r\n").is_err());
assert!(optional_checksum.parse("$GPGGA,data*6A").is_err());

// Lenient: checksum optional, CRLF forbidden
let mut lenient = nmea0183(ChecksumMode::Optional, LineEndingMode::Forbidden)(content_parser);
assert!(lenient.parse("$GPGGA,data*6A").is_ok());
assert!(lenient.parse("$GPGGA,data").is_ok());
assert!(lenient.parse("$GPGGA,data*99").is_err());
assert!(lenient.parse("$GPGGA,data\r\n").is_err());
```

---

## 🧐 How It Works

1. **Framing parser** handles:

   - Start delimiter (`$`)
   - Header (e.g., `GPGGA`)
   - Comma-separated payload
   - Optional checksum (`*CC`)
   - Optional CRLF (`\r\n`)

2. **Your parser** handles:

   - The inner data (`D1,D2,...,Dn`)
   - Can return any type, error, or structure as needed

You get full control over how sentence content is interpreted.

---

## 🦖 Examples

You could parse the data section into fields like this:

```rust
use nmea0183_parser::{ChecksumMode, IResult, LineEndingMode, nmea0183};
use nom::Parser;

fn split_fields(i: &str) -> IResult<&str, Vec<&str>> {
    Ok(("", i.split(',').collect()))
}

let mut parser = nmea0183(ChecksumMode::Required, LineEndingMode::Required)(split_fields);
let (rest, fields) = parser.parse("$GPXXX,field1,field2*4C\r\n").unwrap();
assert_eq!(rest, "");
assert_eq!(fields, vec!["GPXXX", "field1", "field2"]);
```

---

## ❓ FAQ

**Q: Does this parse all NMEA 0183 sentence types like GGA, RMC, etc.?**
🅰️ No — this library helps you _frame and validate_ the sentence. You bring the content parser for specific sentence types.

**Q: What if my device sends non-standard messages?**
🅰️ Configure the parser to relax CRLF or checksum requirements — it’s designed for that.

**Q: Does it allocate?**
🅰️ No heap allocations — parsing is zero-copy where possible.

---

## 🛠️ Contributing

Contributions are very welcome! Open an issue or PR for:

- Bug fixes
- Real-world sentence parsers (GGA, RMC, etc.)
- Integration tests and samples
- Documentation improvements

---

## 📄 License

MIT License © [sdeor](https://github.com/sdeor)
