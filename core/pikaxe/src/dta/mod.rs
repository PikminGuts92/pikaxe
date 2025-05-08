mod errors;
mod io;
mod parser;

pub use errors::*;
pub use io::*;
use parser::*;

const CHAR_NEWLINE: u8 = b'\n';
const CHAR_SPACE: u8 = b' ';

#[derive(Debug)]
pub struct DTAFormat {
    pub use_quoted_symbols: bool, // Ex: 'shortsongname'
    pub indent_char: u8,
    pub indent_char_count: u8,
}

impl Default for DTAFormat {
    fn default() -> Self {
        Self {
            use_quoted_symbols: false,
            indent_char: b' ',
            indent_char_count: 3
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct DataString {
    data: Vec<u8>,
}

impl DataString {
    pub fn from_vec(data: Vec<u8>) -> DataString {
        DataString {
            data,
        }
    }

    pub fn from_string<S: Into<String>>(str: S) -> DataString {
        DataString {
            data: str.into().into_bytes(),
        }
    }

    pub fn as_utf8(&self) -> Option<&str> {
        std::str::from_utf8(&self.data).ok()
    }

    pub fn get_raw(&self) -> &Vec<u8> {
        &self.data
    }
}

impl From<Vec<u8>> for DataString {
    fn from(data: Vec<u8>) -> DataString {
        DataString::from_vec(data)
    }
}

impl From<&[u8]> for DataString {
    fn from(data: &[u8]) -> DataString {
        DataString::from_vec(data.to_vec())
    }
}

impl From<&str> for DataString {
    fn from(data: &str) -> DataString {
        DataString::from_string(data)
    }
}

#[derive(Debug, PartialEq)]
pub enum DataArray {
    Integer(i32),
    Float(f32),
    Variable(DataString),
    //Func() ???
    Object(DataString),
    Symbol(DataString),
    KDataUnhandled,
    IfDef(DataString),
    Else,
    EndIf,
    Array(Vec<DataArray>),
    Command(Vec<DataArray>),
    String(DataString),
    Property(Vec<DataArray>),
    Define(DataString),
    Include(DataString),
    Merge(DataString),
    IfNDef(DataString),
    Autorun,
    Undef(DataString),
}

impl Default for DataArray {
    fn default() -> DataArray {
        DataArray::Integer(0)
    }
}

impl From<i32> for DataArray {
    fn from(value: i32) -> Self {
        DataArray::Integer(value)
    }
}

impl From<f32> for DataArray {
    fn from(value: f32) -> Self {
        DataArray::Float(value)
    }
}

impl DataArray {
    pub fn from<T: Into<DataArray>>(data: T) -> Self {
        data.into()
    }

    pub(crate) fn get_enum_value(&self) -> u32 {
        match self {
            DataArray::Integer(_)     => 0x00,
            DataArray::Float(_)       => 0x01,
            DataArray::Variable(_)    => 0x02,
            //DataArray::Func(_)        => 0x03,
            DataArray::Object(_)      => 0x04,
            DataArray::Symbol(_)      => 0x05,
            DataArray::KDataUnhandled => 0x06,
            DataArray::IfDef(_)       => 0x07,
            DataArray::Else           => 0x08,
            DataArray::EndIf          => 0x09,
            DataArray::Array(_)       => 0x10,
            DataArray::Command(_)     => 0x11,
            DataArray::String(_)      => 0x12,
            DataArray::Property(_)    => 0x13,
            DataArray::Define(_)      => 0x20,
            DataArray::Include(_)     => 0x21,
            DataArray::Merge(_)       => 0x22,
            DataArray::IfNDef(_)      => 0x23,
            DataArray::Autorun        => 0x24,
            DataArray::Undef(_)       => 0x25,
        }
    }

    pub fn find_value_for_symbol<'a, 'b, T: Into<&'b DataString>>(&'a self, symbol: T) -> Option<&[DataArray]> {
        let DataArray::Array(array) = self else {
            return None;
        };

        let symbol = symbol.into();

        match array.split_first() {
            // Found element
            Some((DataArray::Symbol(s) | DataArray::String(s), elements)) if s.eq(symbol) => {
                // Return remaining elements
                return Some(elements);
            },
            // Keep searching
            _ => {}
        }

        // Recursively search
        for val in array {
            let result = val.find_value_for_symbol(symbol);

            if result.is_some() {
                return result;
            }
        }

        None
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            DataArray::Float(f) => Some(*f),
            DataArray::Integer(i) => Some(*i as f32),
            _ => None
        }
    }

    pub fn as_integer(&self) -> Option<i32> {
        match self {
            DataArray::Integer(i) => Some(*i),
            DataArray::Float(f) => Some(*f as i32),
            _ => None
        }
    }

    pub fn as_string(&self) -> Option<&DataString> {
        match self {
            DataArray::Variable(s)
                | DataArray::Object(s)
                | DataArray::Symbol(s)
                | DataArray::IfDef(s)
                | DataArray::String(s)
                | DataArray::Define(s)
                | DataArray::Include(s)
                | DataArray::Merge(s)
                | DataArray::IfNDef(s)
                | DataArray::Undef(s) => Some(s),
            _ => None
        }
    }

    pub fn print<T: std::io::Write>(&self, stream: &mut T) -> Result<(), std::io::Error> {
        self.print_with_format(stream, DTAFormat::default())
    }

    pub fn print_with_format<T: std::io::Write>(&self, stream: &mut T, format: DTAFormat) -> Result<(), std::io::Error> {
        self.write_to_stream(stream, &format, 0).map(|_| ())
    }

    fn write_to_stream<T: std::io::Write>(&self, stream: &mut T, format: &DTAFormat, depth: u32) -> Result<(), std::io::Error> {
        match self {
            DataArray::Integer(i) => {
                write!(stream, "{i}")?;
            },
            DataArray::Float(f) => {
                write!(stream, "{f:?}")?; // At least 1 decimal place...
            },
            DataArray::Variable(v) => {
                stream.write_all(b"$")?;
                stream.write_all(&v.data)?;
            },
            //DataArray::Func(_)        => 0x03,
            DataArray::Object(_o) => {
                todo!("Support object dta serialization")
            },
            DataArray::Symbol(s) => {
                if s.data.iter().any(|c| c.eq(&CHAR_SPACE)) {
                    // Write as string if it contains space
                    stream.write_all(b"\"")?;
                    stream.write_all(&s.data)?;
                    stream.write_all(b"\"")?;
                }
                else if format.use_quoted_symbols {
                    stream.write_all(b"\'")?;
                    stream.write_all(&s.data)?;
                    stream.write_all(b"\'")?;
                } else {
                    stream.write_all(&s.data)?;
                }
            },
            DataArray::KDataUnhandled => {
                stream.write_all(b"kDataUnhandled")?;
            },
            DataArray::IfDef(t) => {
                stream.write_all(b"#ifdef ")?;
                stream.write_all(&t.data)?;
            },
            DataArray::Else => {
                stream.write_all(b"#else")?;
            },
            DataArray::EndIf => {
                stream.write_all(b"#endif")?;
            },
            DataArray::Array(da) => {
                stream.write_all(b"(")?;
                write_elements(stream, da, format, depth, false)?;
                stream.write_all(b")")?;
            },
            DataArray::Command(da) => {
                stream.write_all(b"{")?;
                write_elements(stream, da, format, depth, false)?;
                stream.write_all(b"}")?;
            },
            DataArray::String(s) => {
                // TODO: Escape certain characters...
                stream.write_all(b"\"")?;
                stream.write_all(&s.data)?;
                stream.write_all(b"\"")?;
            },
            DataArray::Property(da) => {
                stream.write_all(b"[")?;
                write_elements(stream, da, format, depth, false)?;
                stream.write_all(b"]")?;
            },
            DataArray::Define(t) => {
                stream.write_all(b"#define ")?;
                stream.write_all(&t.data)?;
            },
            DataArray::Include(t) => {
                stream.write_all(b"#include ")?;
                stream.write_all(&t.data)?;
            },
            DataArray::Merge(t) => {
                stream.write_all(b"#merge ")?;
                stream.write_all(&t.data)?;
            },
            DataArray::IfNDef(t) => {
                stream.write_all(b"#ifndef ")?;
                stream.write_all(&t.data)?;
            },
            DataArray::Autorun => {
                stream.write_all(b"#autorun")?;
            },
            DataArray::Undef(t) => {
                stream.write_all(b"#undef ")?;
                stream.write_all(&t.data)?;
            },
        };

        Ok(())
    }

    fn is_simple_type(&self) -> bool {
        match self {
            DataArray::Integer(_)
                | DataArray::Float(_)
                | DataArray::Variable(_)
                | DataArray::Symbol(_)
                | DataArray::KDataUnhandled => true,
            _ => false
        }
    }
}

#[derive(Debug, Default)]
pub struct RootData {
    pub data: Vec<DataArray>,
}

impl RootData {
    pub fn new() -> RootData {
        RootData::default()
    }

    pub fn print<T: std::io::Write>(&self, stream: &mut T) -> Result<(), std::io::Error> {
        self.print_with_format(stream, DTAFormat::default())
    }

    pub fn print_with_format<T: std::io::Write>(&self, stream: &mut T, format: DTAFormat) -> Result<(), std::io::Error> {
        write_elements(stream, &self.data, &format, 0, true)
    }
}

fn write_elements<T: std::io::Write>(stream: &mut T, elements: &Vec<DataArray>, format: &DTAFormat, depth: u32, is_root: bool) -> Result<(), std::io::Error> {
    let only_simple_types = elements.iter().all(|e| e.is_simple_type());

    // Always write first element without special spacing
    for element in elements.iter().take(1) {
        element.write_to_stream(stream, format, depth)?;
    }

    if only_simple_types {
        // Write as single line
        for element in elements.iter().skip(1) {
            stream.write_all(&[CHAR_SPACE])?;
            element.write_to_stream(stream, format, depth + 1)?;
        }
    } else if !only_simple_types && elements.len() > 1 {
        // Write on multiple lines
        let extra_indent = if is_root { 0 } else { 1 };
        let indent_size = (format.indent_char_count as usize) * (depth as usize + extra_indent);
        let indent = (0..indent_size)
            .map(|_| format.indent_char)
            .collect::<Vec<_>>();

        for element in elements.iter().skip(1) {
            stream.write_all(&[CHAR_NEWLINE])?;
            stream.write_all(&indent)?;
            element.write_to_stream(stream, format, depth + 1)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use super::*;

    #[rstest]
    #[case(DataArray::Integer(500), b"500")]
    #[case(DataArray::Integer(-42), b"-42")]
    #[case(DataArray::Float(0.0), b"0.0")]
    #[case(DataArray::Float(0.3), b"0.3")]
    #[case(DataArray::Float(0.38), b"0.38")]
    #[case(DataArray::Float(-0.45), b"-0.45")]
    #[case(DataArray::Variable("test".into()), b"$test")]
    // TODO: Test case for func + object
    #[case(DataArray::Symbol("test".into()), b"test")]
    #[case(DataArray::Symbol("lol look at these spaces".into()), b"\"lol look at these spaces\"")]
    #[case(DataArray::KDataUnhandled, b"kDataUnhandled")]
    #[case(DataArray::IfDef("something1".into()), b"#ifdef something1")]
    #[case(DataArray::Else, b"#else")]
    #[case(DataArray::EndIf, b"#endif")]
    #[case(DataArray::Array(vec![DataArray::Symbol("version".into()), DataArray::Integer(123)]), b"(version 123)")]
    #[case(DataArray::Array(vec![DataArray::Symbol("name".into()), DataArray::String("Doctor Worm".into())]), b"(name\n   \"Doctor Worm\")")]
    #[case(DataArray::Array(vec![DataArray::Symbol("doctorworm".into()), DataArray::Array(vec![DataArray::Symbol("name".into()), DataArray::String("Doctor Worm".into())])]), b"(doctorworm\n   (name\n      \"Doctor Worm\"))")]
    // TODO: Test case for command
    #[case(DataArray::String("test".into()), b"\"test\"")]
    // TODO: Test case for property
    #[case(DataArray::Define("whatever".into()), b"#define whatever")]
    #[case(DataArray::Include("something.dta".into()), b"#include something.dta")]
    #[case(DataArray::Merge("something.dta".into()), b"#merge something.dta")]
    #[case(DataArray::IfNDef("whatever".into()), b"#ifndef whatever")]
    #[case(DataArray::Autorun, b"#autorun")]
    #[case(DataArray::Undef("whatever".into()), b"#undef whatever")]
    fn print_data_array_test<const N: usize>(#[case] data: DataArray, #[case] expected: &[u8; N]) {
        use std::io::{BufWriter, Write};

        let mut buffer = BufWriter::new(Vec::new());
        data.print(&mut buffer).unwrap();

        let expected_str = std::str::from_utf8(expected).unwrap();
        let buffer_str = std::str::from_utf8(buffer.buffer()).unwrap();

        //assert_eq!(expected, buffer.buffer());
        assert_eq!(expected_str, buffer_str);
    }
}