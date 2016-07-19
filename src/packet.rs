
/* Store a Location, with latitude, longitude, and optional altitude. */
#[derive(Debug, PartialEq)]
pub struct Location {
    pub latitude: f32,
    pub longitude: f32,
    pub altitude: Option<f32>,
}

/* Store a Wind Speed, with speed and optional bearing. */
#[derive(Debug, PartialEq)]
pub struct WindSpeed {
    pub speed: f32,
    pub bearing: Option<f32>,
}

/* Store one of any of the data types */
#[derive(Debug, PartialEq)]
pub enum DataField<'a> {
    Temperature(Vec<f32>),
    Voltage(Vec<f32>),
    Humidity(Vec<f32>),
    Pressure(Vec<f32>),
    Sun(Vec<f32>),
    Rssi(Vec<f32>),
    Count(Vec<f32>),
    Custom(Vec<f32>),
    Location(Location),
    WindSpeed(WindSpeed),
    Zombie(u8),
    Comment(&'a str)
}

/* Store a whole packet */
#[derive(Debug, PartialEq)]
pub struct Packet<'a> {
    pub repeat: u8,
    pub sequence: char,
    pub data: Vec<DataField<'a>>,
    pub path: Vec<&'a str>
}

