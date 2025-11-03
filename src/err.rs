#[allow(dead_code)]
#[derive(Debug)]
pub enum SprinklerError {
    Yaml(serde_yaml::Error),
    Io(std::io::Error),
}

impl From<serde_yaml::Error> for SprinklerError {
    fn from(error: serde_yaml::Error) -> Self {
        SprinklerError::Yaml(error)
    }
}

impl From<std::io::Error> for SprinklerError {
    fn from(error: std::io::Error) -> Self {
        SprinklerError::Io(error)
    }
}
