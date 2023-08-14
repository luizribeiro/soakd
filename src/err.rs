#[derive(Debug)]
pub enum SprinklerError {
    YAML(serde_yaml::Error),
    IO(std::io::Error),
}

impl From<serde_yaml::Error> for SprinklerError {
    fn from(error: serde_yaml::Error) -> Self {
        SprinklerError::YAML(error)
    }
}

impl From<std::io::Error> for SprinklerError {
    fn from(error: std::io::Error) -> Self {
        SprinklerError::IO(error)
    }
}
