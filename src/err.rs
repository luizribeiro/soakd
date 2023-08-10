#[derive(Debug)]
pub enum SprinklerError {
    YAML(serde_yaml::Error),
    IO(std::io::Error),
}
