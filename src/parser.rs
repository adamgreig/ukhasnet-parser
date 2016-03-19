use nom::{digit, alphanumeric};
use std::str::FromStr;

/* Simple parsers for the repeat-count and sequence-number */
named!(pub repeat<&str, u8>,
       map_res!(flat_map!(take_s!(1), digit), FromStr::from_str));
named!(pub sequence<&str, char>,
       map!(flat_map!(take_s!(1), is_a_s!("abcdefghijklmnopqrstuvwxyz")),
            |s: &str| { s.chars().nth(0).unwrap() }));

/* Parse the numeric data common to most fields */
named!(numeric_data<&str, f32>,
       map_res!(is_a_s!("+-1234567890."), FromStr::from_str));

/* Store a Location, with latitude, longitude, and optional altitude. */
#[derive(Debug, PartialEq)]
pub struct Location {
    latitude: f32,
    longitude: f32,
    altitude: Option<f32>,
}

/* Store a Wind Speed, with speed and optional bearing. */
#[derive(Debug, PartialEq)]
pub struct WindSpeed {
    speed: f32,
    bearing: Option<f32>,
}

/* Store one of any of the data types */
#[derive(Debug, PartialEq)]
pub enum DataField<'a> {
    Temperature(Vec<f32>),
    Voltage(Vec<f32>),
    Humidity(Vec<f32>),
    Pressure(Vec<f32>),
    Sun(Vec<f32>),
    RSSI(Vec<f32>),
    Count(Vec<f32>),
    Custom(Vec<f32>),
    Location(Location),
    WindSpeed(WindSpeed),
    Zombie(u8),
    Comment(&'a str)
}

/* Macro to implement the common pattern of a letter followed by one or more
 * numbers separated by commas.
 */
macro_rules! scalar_data_array {
    ($typename:ident, $name:ident, $tag:expr) => {
        named!(pub $name<&str, DataField>, chain!(
            tag_s!($tag) ~
            data: separated_nonempty_list!(tag_s!(","), numeric_data),
            || {DataField::$typename(data)}
        ));
    }
}

/* Generate parsers for the simple types as above. */
scalar_data_array!(Temperature, temperature, "T");
scalar_data_array!(Voltage, voltage, "V");
scalar_data_array!(Humidity, humidity, "H");
scalar_data_array!(Pressure, pressure, "P");
scalar_data_array!(Sun, sun, "S");
scalar_data_array!(RSSI, rssi, "R");
scalar_data_array!(Count, count, "C");
scalar_data_array!(Custom, custom, "X");

/* Parse a Location.
 * Note that (a bug in Nom?) there must be some remaining data after the end
 * of the location if there is no altitude specified.
 */
named!(pub location<&str, DataField>, chain!(
    tag_s!("L") ~
    latitude: numeric_data ~
    tag_s!(",") ~
    longitude: numeric_data ~
    altitude: preceded!(tag_s!(","), numeric_data)?,
    || {DataField::Location(Location{
            latitude: latitude, longitude: longitude, altitude: altitude})}
));

/* Parse a WindSpeed.
 * Note that as with `location`, there must be some remaining data after the
 * end of the wind speed if there is no bearing specified.
 */
named!(pub windspeed<&str, DataField>, chain!(
    tag_s!("W") ~
    speed: numeric_data ~
    bearing: preceded!(tag_s!(","), numeric_data)?,
    || {DataField::WindSpeed(WindSpeed{speed: speed, bearing: bearing})}
));

/* Parse Zombie mode. */
named!(pub zombie<&str, DataField>, chain!(
    tag_s!("Z") ~
    mode: flat_map!(take_s!(1), digit),
    || {DataField::Zombie(mode.parse::<u8>().unwrap())}
));

/* Parse comments/messages */
named!(pub comment<&str, DataField>, chain!(
    tag_s!(":") ~
    comment: is_a_s!("abcdefghijklmnopqrstuvwxyz0123456789+-. "),
    || {DataField::Comment(comment)}));

/* Parse the path at the end of the message */
named!(pub path<&str, Vec<&str> >,
    delimited!(
        tag_s!("["),
        separated_nonempty_list!(tag_s!(","), alphanumeric),
        tag_s!("]")
    )
);

/* Parse the data section of a packet */
named!(pub packet_data<&str, Vec<DataField> >, many1!(alt!(
    temperature | voltage | humidity | pressure | sun | rssi | count |
    custom | location | windspeed | zombie | comment)));

/* Store a whole packet */
#[derive(Debug, PartialEq)]
pub struct Packet<'a> {
    repeat: u8,
    sequence: char,
    data: Vec<DataField<'a> >,
    path: Vec<&'a str>
}

named!(pub packet<&str, Packet>, chain!(
    repeat: repeat ~
    sequence: sequence ~
    data: packet_data ~
    path: path,
    || { Packet{ repeat: repeat, sequence: sequence, data: data, path: path }}
));

#[cfg(test)]
mod tests {
    use super::*;
    use super::numeric_data;
    use nom::IResult::{Done, Error};
    use nom::Err::Position;
    use nom::ErrorKind;

    #[test]
    fn test_numeric_data() {
        assert_eq!(numeric_data("12"), Done("", 12.0));
        assert_eq!(numeric_data("12abc"), Done("abc", 12.0));
        assert_eq!(numeric_data("12.5"), Done("", 12.5));
        assert_eq!(numeric_data("-12.5"), Done("", -12.5));
        assert_eq!(numeric_data("+.5"), Done("", 0.5));
        assert_eq!(numeric_data("1Z"), Done("Z", 1.0));
        assert_eq!(numeric_data("a123"),
                   Error(Position(ErrorKind::IsAStr, "a123")));
    }

    #[test]
    fn test_repeat() {
        assert_eq!(repeat("3abc"), Done("abc", 3));
        assert_eq!(repeat("2345"), Done("345", 2));
        assert_eq!(repeat("a123"), Error(Position(ErrorKind::Digit, "a123")));
        assert_eq!(repeat("!123"), Error(Position(ErrorKind::Digit, "!123")));
    }

    #[test]
    fn test_sequence() {
        assert_eq!(sequence("b123"), Done("123", 'b'));
        assert_eq!(sequence("z123"), Done("123", 'z'));
        assert_eq!(sequence("A12"), Error(Position(ErrorKind::IsAStr, "A12")));
        assert_eq!(sequence("123"), Error(Position(ErrorKind::IsAStr, "123")));
        assert_eq!(sequence("!12"), Error(Position(ErrorKind::IsAStr, "!12")));
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
                   Done("", DataField::RSSI(vec!{-12., -15., -8.})));
        assert_eq!(count("C123"),
                   Done("", DataField::Count(vec!{123.})));
        assert_eq!(custom("X123,4.56"),
                   Done("", DataField::Custom(vec!{123., 4.56})));
        assert_eq!(temperature("Thello"),
                   Error(Position(ErrorKind::IsAStr, "hello")));
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

        assert_eq!(location("L51.52,abc,345"),
                   Error(Position(ErrorKind::IsAStr, "abc,345")));
    }

    #[test]
    fn test_comment() {
        assert_eq!(comment(":hello worldT123"),
                   Done("T123", DataField::Comment("hello world")));

        assert_eq!(comment(":ABC"), Error(Position(ErrorKind::IsAStr, "ABC")));
    }

    #[test]
    fn test_windspeed() {
        assert_eq!(windspeed("W15[]"),
                   Done("[]", DataField::WindSpeed(WindSpeed{
                        speed: 15.0, bearing: None})));

        assert_eq!(windspeed("W15,123"),
                   Done("", DataField::WindSpeed(WindSpeed{
                       speed: 15.0, bearing: Some(123.0)})));

        assert_eq!(windspeed("Whello"),
                   Error(Position(ErrorKind::IsAStr, "hello")));
    }

    #[test]
    fn test_zombie() {
        assert_eq!(zombie("Z0"), Done("", DataField::Zombie(0)));
        assert_eq!(zombie("Z1"), Done("", DataField::Zombie(1)));
        assert_eq!(zombie("Zno"), Error(Position(ErrorKind::Digit, "no")));
    }

    #[test]
    fn test_path() {
        assert_eq!(path("[A,B,C]"), Done("", vec!{"A", "B", "C"}));
        assert_eq!(path("[DH123]"), Done("", vec!{"DH123"}));
        assert_eq!(path("[T/1]"), Error(Position(ErrorKind::TagStr, "/1]")));
    }

    #[test]
    fn test_data() {
        assert_eq!(
            packet_data("T21H68S123X1,2,3:hello[AG]"),
            Done("[AG]", vec!{DataField::Temperature(vec!{21.}),
                              DataField::Humidity(vec!{68.}),
                              DataField::Sun(vec!{123.}),
                              DataField::Custom(vec!{1., 2., 3.}),
                              DataField::Comment("hello")}));
    }

    #[test]
    fn test_packet() {
        assert_eq!(
            packet("3bT21S80[AG,AH]"),
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
    }
}
