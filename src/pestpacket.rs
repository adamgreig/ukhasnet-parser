
/* Store a Location, with latitude, longitude, and optional altitude. */
#[derive(Debug, PartialEq)]
pub struct Location {
    pub latlng: Option<(f32, f32)>,
    pub alt: Option<f32>,
}

/* Store a Wind Speed, with speed and optional bearing. */
#[derive(Debug, PartialEq)]
pub struct WindSpeed {
    pub speed: Option<f32>,
    pub bearing: Option<f32>,
}

/* Store one of any of the data types */
#[derive(Debug, PartialEq)]
pub enum DataField {
    Temperature(Vec<f32>),
    Voltage(Vec<f32>),
    Current(Vec<f32>),
    Humidity(Vec<f32>),
    Pressure(Vec<f32>),
    Sun(Vec<f32>),
    Rssi(Vec<f32>),
    Count(Vec<f32>),
    Custom(Vec<f32>),
    Location(Location),
    WindSpeed(WindSpeed),
    Zombie(u8),
}

/* Store a whole packet */
#[derive(Debug, PartialEq)]
pub struct Packet {
    pub repeat: u8,
    pub sequence: char,
    pub data: Vec<DataField>,
    pub comment: Option<String>,
    pub path: Vec<String>
}
