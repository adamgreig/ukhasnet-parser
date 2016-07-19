use pest::prelude::*;
use pestpacket::{Location, WindSpeed, DataField, Packet};

impl_rdp! {
    grammar! {
        digit       = _{ ['0'..'9'] }
        integer     = @{ (["+"] | ["-"])? ~ digit+ }
        decimal     = @{ (["+"] | ["-"])? ~ digit+ ~ (["."] ~ digit+)? }

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
        zombie      =  { ["Z"] ~ ( ["0"] | ["1"] ) }

        data_field  =  { voltage | temperature | humidity | pressure | custom |
                        sun  | rssi | windspeed | location | count | zombie }

        data        =   { data_field* }

        message_content = @{ (letter | digit | symbol)* }
        message         =  { [":"] ~ message_content }

        node_name_content = @{ (uppercase_letter | digit)* }
        node_name         =  { node_name_content }

        path        =  { ["["] ~ node_name ~ ( [","] ~ node_name )* ~ ["]"] }

        packet      =  { repeat ~ sequence ~ data ~ message? ~ path ~ eoi}
    }

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
            (_, &mode) => mode.parse::<u8>().unwrap()
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

        _message(&self) -> Option<String> {
            (_: message, &message: message_content) =>
                Some(message.to_owned()),
            () => None,
        }

        _node_name(&self) -> String {
            (&name: node_name_content) => name.to_owned()
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
             data: _data(), message: _message(), path: _path()) =>
                Packet{ repeat: repeat, sequence: sequence, data: data,
                        comment: message, path: path }
        }
    }
}
