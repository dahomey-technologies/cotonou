use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum HostingEnvironment {
    Dev,
    Cert,
    Live
}