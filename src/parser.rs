use pest::prelude::*;
use std::error;
use std::fmt;
use packet::{Location, WindSpeed, DataField, Packet};

impl_rdp! {
    /*
     * UKHASnet Packet Grammar
     *
     * Reference: https://github.com/UKHASnet/protocol/blob/master/grammar.ebnf
     * Syntax: http://dragostis.github.io/pest/pest/macro.grammar!.html#syntax
     *
     * In brief: Literals are quoted and in square brackets,
     *           ~ concatenates,
     *           + means "one or more"
     *           * means "zero or more"
     *           ? means "optional"
     *           _ means "this token is silent and is subsumed into the parent"
     */
    grammar! {
        digit       = _{ ['0'..'9'] }
        integer     =  { (["+"] | ["-"])? ~ digit+ }
        decimal     =  { (["+"] | ["-"])? ~ digit+ ~ (["."] ~ digit+)? }

        lowercase_letter = _{ ['a'..'z'] }
        uppercase_letter = _{ ['A'..'Z'] }

        letter      = _{ lowercase_letter | uppercase_letter }
        symbol      = _{
            [" "] | ["!"] | ["\""] | ["#"] | ["$"] | ["%"] | ["&"] | ["'"]  |
            ["("] | [")"] | ["*"]  | ["+"] | [","] | ["-"] | ["."] | ["/"]  |
            [":"] | [";"] | ["<"]  | ["="] | [">"] | ["?"] | ["@"] | ["\\"] |
            ["^"] | ["_"] | ["`"]  | ["{"] | ["|"] | ["}"] | ["~"]
        }

        repeat      =  { digit }
        sequence    =  { lowercase_letter }

        decimal_list=  { decimal? ~ ( [","] ~ decimal? )* }

        voltage     =  { ["V"] ~ decimal_list }
        current     =  { ["I"] ~ decimal_list }
        temperature =  { ["T"] ~ decimal_list }
        humidity    =  { ["H"] ~ decimal_list }
        pressure    =  { ["P"] ~ decimal_list }
        custom      =  { ["X"] ~ decimal_list }
        sun         =  { ["S"] ~ decimal_list }
        rssi        =  { ["R"] ~ decimal_list }
        count       =  { ["C"] ~ decimal_list }
        windspeed   =  { ["W"] ~ decimal? ~ ( [","] ~ decimal? )? }
        location    =  { ["L"] ~ ( (decimal ~ [","] ~ decimal)? | [","] )
                               ~ ( [","] ~ decimal? )? }

        zombie_mode =  { ["0"] | ["1"] }
        zombie      =  { ["Z"] ~ zombie_mode }

        data_field  =  { voltage | current | temperature | humidity |
                         pressure | custom | sun  | rssi | windspeed |
                         location | count | zombie }

        data        =   { data_field* }

        comment_content     =  { (letter | digit | symbol)* }
        comment             =  { [":"] ~ comment_content }

        /* The specification says upper case letters only, but enough deployed
         * nodes use lower case names for it to be an annoying breaking change.
         * To compromise, we will accept lower case letters in node names, but
         * convert them to upper case in the parser, so they're stored and
         * displayed as all upper case thereafter.
         */
        node_name_content   =  { (letter | digit)* }
        node_name           =  { node_name_content }

        path        =  { ["["] ~ node_name ~ ( [","] ~ node_name )* ~ ["]"] }

        packet      =  { repeat ~ sequence ~ data ~ comment? ~ path ~ eoi }
    }

    /*
     * UKHASnet packet parsing
     *
     * Each rule maps one or more tokens from the stream into some reduced
     * type, for example a sequence token into a char, or three decimal tokens
     * into a Location struct.
     *
     * Some rules are called recursively, such as _decimal_list, which first
     * consumes the decimal_list token, then consumes a number of decimals.
     * Think about it backwards: it will recurse into the stack until it cannot
     * match any decimals, at which point it returns a new Vec, then steps back
     * up the stack, inserting into that Vec, until it eventually returns it.
     * _data and _path behave similarly.
     *
     * Note that the extensive unwrap() is fine because the parser itself will
     * have validated that the field contained the relevant type. Any panics
     * would be a bug in the underlying parser library.
     */
    process! {
        _repeat(&self) -> u8 {
            (&repeat: repeat) => repeat.parse::<u8>().unwrap()
        }

        _sequence(&self) -> char {
            (&sequence: sequence) =>
                sequence.chars().nth(0).unwrap()
        }

        _location(&self) -> Location {
            (&latitude: decimal, &longitude: decimal, &altitude: decimal) => {
                let lat = latitude.parse::<f32>().unwrap();
                let lng = longitude.parse::<f32>().unwrap();
                let alt = altitude.parse::<f32>().unwrap();
                Location{ latlng: Some((lat, lng)), alt: Some(alt) }
            },
            (&latitude: decimal, &longitude: decimal) => {
                let lat = latitude.parse::<f32>().unwrap();
                let lng = longitude.parse::<f32>().unwrap();
                Location{ latlng: Some((lat, lng)), alt: None }
            },
            (&altitude: decimal) => {
                let alt = altitude.parse::<f32>().unwrap();
                Location{ latlng: None, alt: Some(alt) }
            },
            () => Location{ latlng: None, alt: None },
        }

        _windspeed(&self) -> WindSpeed {
            (&speed: decimal, &bearing: decimal) => {
                let speed = speed.parse::<f32>().unwrap();
                let bearing = bearing.parse::<f32>().unwrap();
                WindSpeed{speed: Some(speed), bearing: Some(bearing)}
            },
            (&speed: decimal) => {
                let speed = speed.parse::<f32>().unwrap();
                WindSpeed{speed: Some(speed), bearing: None}
            },
            () => WindSpeed{speed: None, bearing: None},
        }

        _decimal_list(&self) -> Vec<f32> {
            (_: decimal_list, list: _decimal_list()) => list,
            (&head: decimal, mut tail: _decimal_list()) => {
                tail.insert(0, head.parse::<f32>().unwrap());
                tail
            },
            () => Vec::<f32>::new(),
        }

        _zombie(&self) -> u8 {
            (&mode: zombie_mode) => mode.parse::<u8>().unwrap()
        }

        _datafield(&self) -> DataField {
            (_: voltage, voltages: _decimal_list()) =>
                DataField::Voltage(voltages),
            (_: current, currents: _decimal_list()) =>
                DataField::Current(currents),
            (_: temperature, temperatures: _decimal_list()) =>
                DataField::Temperature(temperatures),
            (_: humidity, humidities: _decimal_list()) =>
                DataField::Humidity(humidities),
            (_: pressure, pressures: _decimal_list()) =>
                DataField::Pressure(pressures),
            (_: custom, customs: _decimal_list()) =>
                DataField::Custom(customs),
            (_: sun, suns: _decimal_list()) =>
                DataField::Sun(suns),
            (_: rssi, rssis: _decimal_list()) =>
                DataField::Rssi(rssis),
            (_: count, counts: _decimal_list()) =>
                DataField::Count(counts),
            (_: windspeed, windspeed: _windspeed()) =>
                DataField::WindSpeed(windspeed),
            (_: location, location: _location()) =>
                DataField::Location(location),
            (_: zombie, zombie: _zombie()) =>
                DataField::Zombie(zombie),
        }

        _data(&self) -> Vec<DataField> {
            (_: data, fields: _data()) => fields,
            (_: data_field, head: _datafield(), mut tail: _data()) => {
                tail.insert(0, head);
                tail
            },
            () => Vec::<DataField>::new(),
        }

        _comment(&self) -> Option<String> {
            (_: comment, &comment: comment_content) =>
                Some(comment.to_owned()),
            () => None,
        }

        _node_name(&self) -> String {
            (&name: node_name_content) => name.to_owned().to_uppercase()
        }

        _path(&self) -> Vec<String> {
            (_: path, names: _path()) => names,
            (_: node_name, head: _node_name(), mut tail: _path()) => {
                tail.insert(0, head);
                tail
            },
            () => Vec::<String>::new(),
        }

        parse(&self) -> Packet {
            (_: packet, repeat: _repeat(), sequence: _sequence(),
             data: _data(), comment: _comment(), path: _path()) =>
                Packet{ repeat: repeat, sequence: sequence, data: data,
                        comment: comment, path: path }
        }
    }
}

/// Contains the position in the input at which a parsing error occurred,
/// and a Vec of token names we expected to see instead.
#[derive(Debug)]
pub struct ParserError {
    pub position: usize,
    pub expected: Vec<String>,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parser error at position {}", self.position)
    }
}

impl error::Error for ParserError {
    fn description(&self) -> &str { "Parser error" }
    fn cause(&self) -> Option<&error::Error> { None }
}

impl ParserError {
    /// Extract error information from a parser.
    pub fn from_parser<'a, T: Input<'a>>(parser: &mut Rdp<T>) -> ParserError {
        let (expected, position) = parser.expected();
        let exp = expected.iter().map(|r| { format!("{:?}", r) }).collect();
        ParserError{ position: position, expected: exp }
    }
}
