use nom::*;
use nom::branch::{alt, permutation};
use nom::bytes::complete::{is_not, tag, take, take_till, take_till1, take_while, take_while1};
use nom::character::{is_alphanumeric, is_digit, is_hex_digit};
use nom::character::complete::{alpha1, alphanumeric0, alphanumeric1, digit1, hex_digit1, one_of};
use nom::combinator::{all_consuming, map, map_parser, map_res, not, opt, recognize};
use nom::error::{context, Error};
use nom::multi::{many0, separated_list0};
use nom::number::complete::recognize_float;
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use super::{DataArray, ParseDTAError, RootData, DataString};
use std::num::{IntErrorKind, ParseIntError};

const WS_CHARACTERS: &[u8] = b" \t\r\n\x0c";
const SPACE_CHARACTERS: &[u8] = b" \t";
const NEWLINE_CHARACTERS: &[u8] = b"\r\n";
const DIGIT_CHARACTERS_10: &[u8] = b"0123456789";
const DIGIT_CHARACTERS_16: &[u8] = b"0123456789AaBbCcDdEeFf";

const OPEN_BRACKET: &[u8] = b"(";
const CLOSE_BRACKET:&[u8] = b")";

const QUOTE_CHARACTER: &[u8] = b"\"";
const COMMENT_CHARACTER: &[u8] = b";";

pub struct ParsedSong<'a> {
    pub start: usize,
    pub size: usize,
    pub data: &'a [u8],
}

pub struct DTAParser<'a> {
    pub depth: usize,
    pub line_number: usize,
    pub char_index: usize,
    pub data_array: Vec<(DataArray, usize)>, // Includes char start index
    pub remaining: &'a [u8]
}

impl<'a> DTAParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            depth: 0,
            line_number: 0,
            char_index: 0,
            data_array: Vec::new(),
            remaining: data
        }
    }

    pub fn parse(mut self) {
        self.parse_array();
    }

    fn parse_array(&mut self) -> Vec<(DataArray, usize)> {
        let data = Vec::new();

        self.consume_whitespace();
        // TODO: Consume comments

        data
    }

    fn consume_whitespace(&mut self) {
        let mut chars_moved = 0;
        let mut lines_moved = 0;

        for c in self.remaining.iter() {
            match c {
                b'\n' => {
                    chars_moved += 1;
                    lines_moved += 1;
                },
                c if c.is_ascii_whitespace() => {
                    chars_moved += 1;
                }
                _ => {
                    break;
                }
            }
        }

        // Update self
        self.char_index += chars_moved;
        self.line_number += lines_moved;
        self.remaining = &self.remaining[chars_moved..];
    }
}

/*pub(crate) fn parse_dta_str<'a>(text: &'a [u8]) -> Vec<DataArray> {
    //let mut parse_info = 

    Vec::new()
}*/

fn take_ws<'a>(text: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    take_while(move |c| WS_CHARACTERS.contains(&c))(text)
}

fn take_newline<'a>(text: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    take_while(move |c| NEWLINE_CHARACTERS.contains(&c))(text)
}

fn take_ws_or_comment<'a>(text: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    let mut ws_result = take_ws(text);

    while let Ok((text, _)) = ws_result {
        let comment_result = take_comment(text);

        if comment_result.is_err() {
            break;
        }

        let (t1, _) = comment_result.unwrap();
        ws_result = take_ws(t1);
    }

    ws_result

    //alt((take_ws, take_comment))(text)
}

fn take_until_newline<'a>(text: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    take_till(move |c| NEWLINE_CHARACTERS.contains(&c))(text)
}

fn take_comment<'a>(text: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    context(
        "comment",
        preceded(
            tag(COMMENT_CHARACTER),
            preceded(take_until_newline, take_newline)
        )
    )(text)
}

fn take_section<'a>(text: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    delimited(
        preceded(take_ws, tag(OPEN_BRACKET)),
        tag(b"test"), // alt
        preceded(take_ws, tag(CLOSE_BRACKET)),
    )(text)
}

fn take_quoted_string<'a>(text: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    // TODO: Better support escaped characters?
    context(
        "string",
        delimited(
            tag(QUOTE_CHARACTER),
            take_till(move |c| QUOTE_CHARACTER.contains(&c)),
            tag(QUOTE_CHARACTER),
        )
    )(text)
}

fn take_node<'a>(text: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    let (mut r1, _) = take_ws_or_comment(text)?; // Whitespace

    // Read open bracket
    let open_bracket_res: IResult<&'a [u8], &'a [u8]> = tag(OPEN_BRACKET)(r1);
    if open_bracket_res.is_ok() {
        // Nested data...
        r1 = &r1[1..];

        let (nested_r1, _) = take_node(r1)?;
        r1 = nested_r1;
    }

    // Read until close bracket
    let mut close_bracket_res: IResult<&'a [u8], &'a [u8]>;

    loop {
        close_bracket_res = tag(CLOSE_BRACKET)(r1);

        if r1.is_empty() {
            break;
        } else if close_bracket_res.is_ok() {
            r1 = &r1[1..];
            break;
        }


    }

    /*while let Ok((nested_r1, _)) = close_bracket_res {
        /*let comment_result = take_comment(text);

        if comment_result.is_err() {
            break;
        }

        let (t1, _) = comment_result.unwrap();
        ws_result = take_ws(t1);*/
    }*/

    /*let tt: IResult<&'a [u8], &'a [u8]> = alt((
        map(take_quoted_string, |_| (r1, r2))
    ))(r1);*/

    //return Ok((r1, r2));

    //return tt;

    preceded(
        take_ws_or_comment,
        // string
        // int
        // float
        // node
        delimited(
            tag(OPEN_BRACKET),
            preceded(
                take_ws_or_comment,
                take_while(|c| !CLOSE_BRACKET.contains(&c))
            ),
            preceded(
                take_ws_or_comment,
                tag(CLOSE_BRACKET)
            )
        )
    )(text)
}

fn take_root_node<'a>(text: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    preceded(
        take_ws_or_comment,
        // string
        // int
        // float
        // node
        delimited(
            tag(OPEN_BRACKET),
            preceded(
                take_ws_or_comment,
                take_while(|c| !CLOSE_BRACKET.contains(&c))
            ),
            preceded(
                take_ws_or_comment,
                tag(CLOSE_BRACKET)
            )
        )
    )(text)
}

fn parse_string<'a>(text: &'a [u8]) -> IResult<&'a [u8], DataArray> {
    delimited(
        tag(b"\""),
        map(
            take_while(|c: u8| c.ne(&b"\""[0])),
            |s: &'a [u8]| DataArray::String(DataString::from_vec(s.to_owned()))
        ),
        tag(b"\"")
    )(text)
}

fn parse_symbol_name<'a>(text: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    map(
        take_while1(|c: u8| is_alphanumeric(c) || b"._/".contains(&c)),
        |name: &'a [u8]| name,
    )(text)
}

fn parse_symbol<'a>(text: &'a [u8]) -> IResult<&'a [u8], DataArray> {
    all_consuming(
        map(
            alt((
                parse_symbol_name,
                delimited(
                    tag(b"'"),
                    parse_symbol_name,
                    tag(b"'")
                )
            )),
            |symbol: &'a [u8]| DataArray::Symbol(DataString::from_vec(
                symbol.to_vec()
            ))
        )
    )(text)
}

fn parse_int<'a>(text: &'a [u8]) -> IResult<&'a [u8], DataArray> {
    // TODO: Handle under/overflow?
    alt((
        // Base 16
        map_res(
            preceded(
                alt((tag("0x"), tag("0X"))),
                all_consuming(hex_digit1),
            ),
            |num: &'a [u8]| i32::from_str_radix(std::str::from_utf8(num).unwrap(), 16) // Shouldn't fail
                .map(|n| DataArray::Integer(n))
        ),
        // Base 10
        map_res(
            recognize(
                pair(
                    opt(tag("-")),
                    all_consuming(digit1)
                )
            ),
            |num: &'a [u8]| std::str::from_utf8(num).unwrap() // Shouldn't fail
                .parse::<i32>()
                .map(|n| DataArray::Integer(n))
        )
    ))(text)
}

fn parse_float<'a>(text: &'a [u8]) -> IResult<&'a [u8], DataArray> {
    map_res(
        all_consuming(recognize_float),
        |f: &'a [u8]| std::str::from_utf8(f).unwrap()
            .parse::<f32>()
            .map(|f| DataArray::Float(f))
    )(text)
}

fn parse_var_name<'a>(text: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    all_consuming(
        recognize(
            pair(
                alt((
                    alpha1,
                    tag("_")
                )),
                take_while(|c: u8| is_alphanumeric(c) || c.eq(&b'_'))
            )
        )
    )(text)
}

fn parse_variable<'a>(text: &'a [u8]) -> IResult<&'a [u8], DataArray> {
    map(
        all_consuming(
            recognize(
                pair(
                    tag("$"),
                    parse_var_name
                )
            )
        ),
        |data: &'a [u8]| DataArray::Variable(DataString::from_vec(data.to_vec()))
    )(text)
}

fn parse_kdata_unhandled<'a>(text: &'a [u8]) -> IResult<&'a [u8], DataArray> {
    map(
        all_consuming(tag("kDataUnhandled")),
        |_data: &'a [u8]| DataArray::KDataUnhandled
    )(text)
}

/*fn map_int(text: &[u8]) -> Result<DataArray, ParseIntError> {
    let num = match text {
        // Base 16
        [b'0', b'X' | b'x', hd @ ..] if hd.iter().all(|c| is_hex_digit(*c))
            => i32::from_str_radix(std::str::from_utf8(hd).unwrap(), 16),
        // Base 10
        d if d.iter().enumerate().all(|(i, c)| is_digit(*c) || i == 0 && c.eq(&b'-'))
            => std::str::from_utf8(d).unwrap().parse::<i32>(),
        _ => Err(ParseIntError {
            kind: IntErrorKind::InvalidDigit
        })
    };
    
    num.map(|n| DataArray::Integer(n))
}*/

fn take_until_ws_comment_array1<'a>(text: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    take_till1(
        |c: u8| WS_CHARACTERS.contains(&c)
            || b"()[]{};".contains(&c)
    )(text)
}

fn parse_node<'a>(text: &'a [u8]) -> IResult<&'a [u8], DataArray> {
    preceded(
        take_ws_or_comment,
        alt((
            map_parser(
                take_until_ws_comment_array1,
                alt((
                    // Specific keywords...
                    parse_kdata_unhandled,

                    // Int
                    parse_int,
                    // Float
                    parse_float,
                    // Variable
                    parse_variable,
                    // Symbol
                    parse_symbol,
                ))
            ),
            // String
            parse_string,
            // Array
            delimited(
                tag(OPEN_BRACKET),
                map(
                    parse_data_array,
                    |items: Vec<DataArray>| DataArray::Array(items)
                ),
                preceded(
                    take_ws_or_comment,
                    tag(CLOSE_BRACKET)
                )
            ),
            // Property
            delimited(
                tag("["),
                map(
                    parse_data_array,
                    |items: Vec<DataArray>| DataArray::Property(items)
                ),
                preceded(
                    take_ws_or_comment,
                    tag("]")
                )
            ),
            // Command
            delimited(
                tag("{"),
                map(
                    parse_data_array,
                    |items: Vec<DataArray>| DataArray::Command(items)
                ),
                preceded(
                    take_ws_or_comment,
                    tag("}")
                )
            )
        ))
    )(text)
}

fn parse_data_array<'a>(text: &'a [u8]) -> IResult<&'a [u8], Vec<DataArray>> {
    many0(
        parse_node
    )(text)

    /*preceded(
        take_ws_or_comment,
        // string
        // int
        // float
        // node
        delimited(
            tag(OPEN_BRACKET),
            preceded(
                take_ws_or_comment,

                take_while(|c| !CLOSE_BRACKET.contains(&c))
            ),
            preceded(
                take_ws_or_comment,
                tag(CLOSE_BRACKET)
            )
        )
    )(text)*/
}

pub fn parse_dta<'a>(dta: &'a[u8]) -> Result<Vec<ParsedSong>, ParseDTAError> {
    let (r1, r2) = take_root_node(dta)
        .map_err(|_| ParseDTAError::UnknownDTAParseError)?;

    let r1_str = std::str::from_utf8(r1).unwrap();
    let r2_str = std::str::from_utf8(r2).unwrap();

    println!("r1: {r1_str}");
    println!("r2: {r2_str}");

    let (_, items) = parse_data_array(dta).unwrap();

    println!("\n\n{}", String::from_utf8(dta.to_owned()).unwrap());
    println!("{items:#?}");

    //let result = take_section(dta).map_err(|_| ParseDTAError::UnknownDTAParseError)?;

    /*let result = map(
        many0(take_section),
        |songs| songs
    )(dta)
        .map_err(|_| ParseDTAError::UnknownDTAParseError)?;*/

    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use super::*;

    //const DTA_PATH: Option<&str> = option_env!("PIKAXE_TEST_DTA_PATH");

    /*#[rstest]
    #[case(b";whatever \n (wildhoneypie)\n(temporarysecretary\n   (name \"Temporary Secretary\")\n)", 2)] // b"(wildhoneypie)\n(temporarysecretary\n   (name \"Temporary Secretary\")\n)")
    fn parse_dta_test<const N: usize>(#[case] dta: &[u8; N], #[case] expected_count: usize) {
        //let dta_path = DTA_PATH;
        /*match DTA_PATH {
            Some(dta_path) => println!("DTA path is \"{dta_path}\""),
            None => println!("DTA path not found!"),
        };*/

        //let dta_str = std::str::from_utf8(dta).unwrap();

        let result = parse_dta(dta);
        assert!(result.is_ok());

        if let Ok(songs) = result {
            assert_eq!(expected_count, songs.len());
        }
    }*/

    #[rstest]
    #[case(b"", None)]
    #[case(b"   ", None)]
    #[case(b"; Stupid comment", None)]
    #[case(b"5", Some(DataArray::Integer(5)))]
    #[case(b"5.0", Some(DataArray::Float(5.0)))]
    #[case(b"-42.69", Some(DataArray::Float(-42.69)))]
    #[case(b"-100", Some(DataArray::Integer(-100)))]
    #[case(b"0xff", Some(DataArray::Integer(0xFF)))]
    #[case(b"0xFF", Some(DataArray::Integer(0xFF)))]
    #[case(b"\"Test\"", Some(DataArray::String(DataString::from_string("Test"))))]
    #[case(b"\'Test\'", Some(DataArray::Symbol(DataString::from_string("Test"))))]
    #[case(b"Test", Some(DataArray::Symbol(DataString::from_string("Test"))))]
    #[case(b"$test", Some(DataArray::Variable(DataString::from_string("$test"))))]
    #[case(b"$test_song", Some(DataArray::Variable(DataString::from_string("$test_song"))))]
    #[case(b"$p9director", Some(DataArray::Variable(DataString::from_string("$p9director"))))]
    #[case(b"$p9director_1985", Some(DataArray::Variable(DataString::from_string("$p9director_1985"))))]
    #[case(b"$", None)]
    #[case(b"$0", None)]
    #[case(b"$01234", None)]
    #[case(b"$0abc", None)]
    #[case(b"kDataUnhandled", Some(DataArray::KDataUnhandled))]
    fn parse_node_test<const N: usize>(#[case] data: &[u8; N], #[case] expected: Option<DataArray>) {
        let result = parse_node(data)
            .map(|(_, arr)| arr)
            .ok();

        // TODO: Verify expected exception too?

        assert_eq!(expected, result);
    }

    #[rstest]
    #[case(b"(year 2002)", DataArray::Array(vec![DataArray::Symbol(DataString::from_string("year")), DataArray::Integer(2002) ]))]
    fn parse_node_array_test<const N: usize>(#[case] data: &[u8; N], #[case] expected: DataArray) {
        let result = parse_node(data)
            .map(|(_, arr)| arr)
            .unwrap();

        assert_eq!(expected, result);
    }

    #[rstest]
    #[case(b"", None)]
    #[case(b"0", Some(0))]
    #[case(b"1234", Some(1234))]
    #[case(b"-5", Some(-5))]
    #[case(b"2147483647", Some(2147483647))]
    #[case(b"2147483648", None)] // i32::MAX + 1
    #[case(b"-2147483648", Some(-2147483648))]
    #[case(b"-2147483649", None)] // i32::MIN - 1
    #[case(b"0xFF", Some(0xFF))]
    #[case(b"0x1234", Some(0x1234))]
    #[case(b"0xAB00", Some(0xAB00))]
    #[case(b"0xGG", None)]
    fn parse_int_test<const N: usize>(#[case] data: &[u8; N], #[case] expected: Option<i32>) {
        let result = parse_int(data)
            .map(|(_, arr)| arr)
            .ok();

        // TODO: Verify expected exception too?

        assert_eq!(expected.map(|i| DataArray::Integer(i)), result);
    }
}