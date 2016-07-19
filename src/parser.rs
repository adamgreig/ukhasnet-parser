use nom::{digit, alphanumeric, IResult, ErrorKind};
use nom::ErrorKind::{IsAStr, Custom};
use nom::Err::Position;
use std::str::FromStr;

use packet::{Location, WindSpeed, DataField, Packet};

pub use nom::IResult::{Done, Error, Incomplete};

/* Possible error states */
#[derive(Debug, PartialEq)]
pub enum ParseError {
    RepeatNotNumeric,
    SequenceNotAlphabetic,
    DataNotNumeric,
    UnknownType,
    InvalidDataField,
    InvalidLocation,
    InvalidWindSpeed,
    InvalidZombie,
    InvalidComment,
    InvalidCommentCharacter,
    InvalidPath,
    InvalidPathCharacter,
    InvalidData,
    InvalidPacket,
}

/* Custom is_a_s until nom PR#239 is merged */
macro_rules! is_a_s (
  ($input:expr, $arr:expr) => (
    {
      use std::collections::HashSet;
      let set: HashSet<char> = $arr.chars().collect();
      let mut offset = $input.len();
      for (o, c) in $input.char_indices() {
        if !set.contains(&c) {
          offset = o;
          break;
        }
      }
      let res: IResult<_,_> = if offset == 0 {
        Error(Position(IsAStr,$input))
      } else if offset < $input.len() {
        Done(&$input[offset..], &$input[..offset])
      } else {
        Done("", $input)
      };
      res
    }
  );
);

/* Type specialised fix_error macro */
macro_rules! fix (
    ($i:expr, $submac:ident!( $($args:tt)* )) => (
        fix_error!($i, ParseError, $submac!($($args)*))
    );
    ($i:expr, $f:expr) => (
        fix!($i, call!($f));
    );
);


/* Simple parsers for the repeat-count and sequence-number */
named!(repeat<&str, u8, ParseError>,
       error!(Custom(ParseError::RepeatNotNumeric), fix!(
       map_res!(flat_map!(take_s!(1), digit), FromStr::from_str))));
named!(sequence<&str, char, ParseError>,
       error!(Custom(ParseError::SequenceNotAlphabetic), fix!(
       map!(flat_map!(take_s!(1), is_a_s!("abcdefghijklmnopqrstuvwxyz")),
            |s: &str| { s.chars().nth(0).unwrap() }))));

/* Parse the numeric data common to most fields */
named!(numeric_data<&str, f32, ParseError>,
       error!(Custom(ParseError::DataNotNumeric), fix!(
       map_res!(is_a_s!("+-1234567890."), FromStr::from_str))));

/* Macro to implement the common pattern of a letter followed by one or more
 * numbers separated by commas.
 */
macro_rules! scalar_data_array {
    ($typename:ident, $name:ident, $tag:expr) => {
        named!($name<&str, DataField, ParseError>,
        error!(Custom(ParseError::InvalidDataField),
        chain!(
            fix!(tag_s!($tag)) ~
            data: separated_nonempty_list!(fix!(tag_s!(",")), numeric_data),
            || {DataField::$typename(data)}
        )));
    }
}

/* Generate parsers for the simple types as above. */
scalar_data_array!(Temperature, temperature, "T");
scalar_data_array!(Voltage, voltage, "V");
scalar_data_array!(Humidity, humidity, "H");
scalar_data_array!(Pressure, pressure, "P");
scalar_data_array!(Sun, sun, "S");
scalar_data_array!(Rssi, rssi, "R");
scalar_data_array!(Count, count, "C");
scalar_data_array!(Custom, custom, "X");

/* Parse a Location.
 * Note that (a bug in Nom?) there must be some remaining data after the end
 * of the location if there is no altitude specified.
 */
named!(location<&str, DataField, ParseError>,
    error!(Custom(ParseError::InvalidLocation),
    chain!(
        fix!(tag_s!("L")) ~
        latitude: numeric_data ~
        fix!(tag_s!(",")) ~
        longitude: numeric_data ~
        altitude: preceded!(fix!(tag_s!(",")), numeric_data)?,
        || {DataField::Location(Location{
                latitude: latitude, longitude: longitude, altitude: altitude})}
)));

/* Parse a WindSpeed.
 * Note that as with `location`, there must be some remaining data after the
 * end of the wind speed if there is no bearing specified.
 */
named!(windspeed<&str, DataField, ParseError>,
    error!(Custom(ParseError::InvalidWindSpeed),
    chain!(
        fix!(tag_s!("W")) ~
        speed: numeric_data ~
        bearing: preceded!(fix!(tag_s!(",")), numeric_data)?,
        || {DataField::WindSpeed(WindSpeed{speed: speed, bearing: bearing})}
)));

/* Parse Zombie mode. */
named!(zombie<&str, DataField, ParseError>,
    error!(Custom(ParseError::InvalidZombie), fix!(
    chain!(
        tag_s!("Z") ~
        mode: flat_map!(take_s!(1), is_a_s!("01")),
        || {DataField::Zombie(mode.parse::<u8>().unwrap())}
))));

/* Parse comments/messages */
named!(comment<&str, DataField, ParseError>,
    error!(Custom(ParseError::InvalidComment),
    chain!(
        fix!(tag_s!(":")) ~
        comment: error!(Custom(ParseError::InvalidCommentCharacter),
            fix!(is_a_s!("abcdefghijklmnopqrstuvwxyz0123456789+-. "))),
        || {DataField::Comment(comment)}
)));

/* Parse the path at the end of the message */
named!(path<&str, Vec<&str>, ParseError>,
    error!(Custom(ParseError::InvalidPath),
    delimited!(
        fix!(tag_s!("[")),
        fix!(separated_nonempty_list!(tag_s!(","), alphanumeric)),
        error!(Custom(ParseError::InvalidPathCharacter), fix!(tag_s!("]")))
    )
));

/* Parse the data section of a packet up to the start of the path */
named!(packet_data<&str, Vec<DataField>, ParseError>,
    error!(Custom(ParseError::InvalidData),
        many0!(
                dbg!(switch!(fix!(peek!(take_s!(1))),
                    "T" => call!(temperature)   |
                    "V" => call!(voltage)       |
                    "H" => call!(humidity)      |
                    "P" => call!(pressure)      |
                    "S" => call!(sun)           |
                    "R" => call!(rssi)          |
                    "C" => call!(count)         |
                    "X" => call!(custom)        |
                    "L" => call!(location)      |
                    "W" => call!(windspeed)     |
                    "Z" => call!(zombie)        |
                    ":" => call!(comment)
                )
            )
        )
    )
);

/* Parse an entire packet */
named!(pub parse<&str, Packet, ParseError>,
    error!(Custom(ParseError::InvalidPacket),
    chain!(
        repeat: repeat ~
        sequence: sequence ~
        data: packet_data ~
        path: path,
        || {
            Packet{repeat: repeat, sequence: sequence, data: data, path: path}
        }
)));

#[cfg(test)]
mod tests {
    use super::*;
    use super::{repeat, sequence, numeric_data, temperature, voltage, humidity,
                pressure, sun, rssi, count, custom, location, windspeed,
                zombie, comment, path, packet_data};
    use nom::Err::{Position, NodePosition};
    use nom::ErrorKind::{Custom, Fix};

    #[test]
    fn test_numeric_data() {
        assert_eq!(numeric_data("12"), Done("", 12.0));
        assert_eq!(numeric_data("12abc"), Done("abc", 12.0));
        assert_eq!(numeric_data("12.5"), Done("", 12.5));
        assert_eq!(numeric_data("-12.5"), Done("", -12.5));
        assert_eq!(numeric_data("+.5"), Done("", 0.5));
        assert_eq!(numeric_data("1Z"), Done("Z", 1.0));
        assert_eq!(numeric_data("a123"), Error(NodePosition(
            Custom(ParseError::DataNotNumeric), "a123",
            Box::new(Position(Fix, "a123")))));
    }

    #[test]
    fn test_repeat() {
        assert_eq!(repeat("3abc"), Done("abc", 3));
        assert_eq!(repeat("2345"), Done("345", 2));
        assert_eq!(repeat("a123"),
            Error(NodePosition(Custom(ParseError::RepeatNotNumeric), "a123",
                               Box::new(Position(Fix, "a123")))));
        assert_eq!(repeat("!123"),
            Error(NodePosition(Custom(ParseError::RepeatNotNumeric), "!123",
                               Box::new(Position(Fix, "!123")))));
    }

    #[test]
    fn test_sequence() {
        assert_eq!(sequence("b123"), Done("123", 'b'));
        assert_eq!(sequence("z123"), Done("123", 'z'));
        assert_eq!(sequence("A1"),
            Error(NodePosition(Custom(ParseError::SequenceNotAlphabetic), "A1",
                  Box::new(Position(Fix, "A1")))));
        assert_eq!(sequence("12"),
            Error(NodePosition(Custom(ParseError::SequenceNotAlphabetic), "12",
                  Box::new(Position(Fix, "12")))));
        assert_eq!(sequence("!1"),
            Error(NodePosition(Custom(ParseError::SequenceNotAlphabetic), "!1",
                  Box::new(Position(Fix, "!1")))));
    }

    #[test]
    fn test_scalar_arrays() {
        assert_eq!(temperature("T5"),
                   Done("", DataField::Temperature(vec!{5.})));
        assert_eq!(temperature("T12.5"),
                   Done("", DataField::Temperature(vec!{12.5})));
        assert_eq!(temperature("T12.5,-15,8"),
                   Done("", DataField::Temperature(vec!{12.5, -15., 8.})));
        assert_eq!(voltage("V12.5,-15,8"),
                   Done("", DataField::Voltage(vec!{12.5, -15., 8.})));
        assert_eq!(humidity("H12.5,-15,8"),
                   Done("", DataField::Humidity(vec!{12.5, -15., 8.})));
        assert_eq!(pressure("P12.5,-15,8"),
                   Done("", DataField::Pressure(vec!{12.5, -15., 8.})));
        assert_eq!(sun("S12.5,-15,8"),
                   Done("", DataField::Sun(vec!{12.5, -15., 8.})));
        assert_eq!(rssi("R-12,-15,-8"),
                   Done("", DataField::Rssi(vec!{-12., -15., -8.})));
        assert_eq!(count("C123"),
                   Done("", DataField::Count(vec!{123.})));
        assert_eq!(custom("X123,4.56"),
                   Done("", DataField::Custom(vec!{123., 4.56})));

        assert_eq!(temperature("Thello"), Error(
            NodePosition(Custom(ParseError::InvalidDataField), "Thello",
            Box::new(NodePosition(Custom(ParseError::DataNotNumeric), "hello",
            Box::new(Position(Fix, "hello")))))));
    }

    #[test]
    fn test_location() {
        assert_eq!(location("L51.52,-1.23[]"),
                   Done("[]",
                        DataField::Location(
                            Location{
                                latitude: 51.52,
                                longitude: -1.23,
                                altitude: None})));

        assert_eq!(location("L51.52,-1.23,345"),
                   Done("",
                        DataField::Location(
                            Location{
                                latitude: 51.52,
                                longitude: -1.23,
                                altitude: Some(345.0)})));

        assert_eq!(location("L51.52,abc,34"), Error(
            NodePosition(Custom(ParseError::InvalidLocation), "L51.52,abc,34",
            Box::new(NodePosition(Custom(ParseError::DataNotNumeric), "abc,34",
            Box::new(Position(Fix, "abc,34")))))));
    }

    #[test]
    fn test_comment() {
        assert_eq!(comment(":hello worldT123"),
                   Done("T123", DataField::Comment("hello world")));

        assert_eq!(comment(":Hello"), Error(
            NodePosition(Custom(ParseError::InvalidComment), ":Hello",
            Box::new(NodePosition(Custom(ParseError::InvalidCommentCharacter),
                                  "Hello",
            Box::new(Position(Fix, "Hello")))))));
    }

    #[test]
    fn test_windspeed() {
        assert_eq!(windspeed("W15[]"),
                   Done("[]", DataField::WindSpeed(WindSpeed{
                        speed: 15.0, bearing: None})));

        assert_eq!(windspeed("W15,123"),
                   Done("", DataField::WindSpeed(WindSpeed{
                       speed: 15.0, bearing: Some(123.0)})));

        assert_eq!(windspeed("Whello"), Error(
            NodePosition(Custom(ParseError::InvalidWindSpeed), "Whello",
            Box::new(NodePosition(Custom(ParseError::DataNotNumeric), "hello",
            Box::new(Position(Fix, "hello")))))));
    }

    #[test]
    fn test_zombie() {
        assert_eq!(zombie("Z0"), Done("", DataField::Zombie(0)));
        assert_eq!(zombie("Z1"), Done("", DataField::Zombie(1)));
        assert_eq!(zombie("Zno"), Error(
            NodePosition(Custom(ParseError::InvalidZombie), "Zno",
            Box::new(Position(Fix, "no")))));
    }

    #[test]
    fn test_path() {
        assert_eq!(path("[A,B,C]"), Done("", vec!{"A", "B", "C"}));
        assert_eq!(path("[DH123]"), Done("", vec!{"DH123"}));
        assert_eq!(path("[A,B/1]"), Error(
            NodePosition(Custom(ParseError::InvalidPath), "[A,B/1]",
            Box::new(NodePosition(Custom(ParseError::InvalidPathCharacter),
                                  "/1]",
            Box::new(Position(Fix, "/1]")))))));
    }

    #[test]
    fn test_data() {
        assert_eq!(packet_data("T21H68S123X1,2,3:hello[AG]"),
            Done("[AG]", vec!{DataField::Temperature(vec!{21.}),
                              DataField::Humidity(vec!{68.}),
                              DataField::Sun(vec!{123.}),
                              DataField::Custom(vec!{1., 2., 3.}),
                              DataField::Comment("hello")}));
        //assert_eq!(packet_data("T1HaX2[A]"),
            //Done("", vec!{DataField::Temperature(vec!{21.}),
                              //DataField::Humidity(vec!{68.}),
                              //DataField::Sun(vec!{123.}),
                              //DataField::Custom(vec!{1., 2., 3.}),
                              //DataField::Comment("hello")}));
        //assert_eq!(packet_data("T21J5X1"), Error(
            //Position(Custom(ParseError::InvalidData), "T21J5X1")));
    }

    #[test]
    fn test_packet() {
        assert_eq!(parse("3bT21S80[AG,AH]"),
            Done("",
                 Packet {
                     repeat: 3,
                     sequence: 'b',
                     data: vec!{
                         DataField::Temperature(vec!{21.}),
                         DataField::Sun(vec!{80.})
                    },
                    path: vec!{"AG", "AH"}
                 }
            )
        );

        //assert_eq!(parse("3bT1J2X3[A,B]"), Error(
            //Position(Custom(ParseError::InvalidPacket), "J2X3[A,B]")
        //));
    }
}
