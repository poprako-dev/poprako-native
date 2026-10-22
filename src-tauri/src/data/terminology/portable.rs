use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortableTerm {
    pub source: String,
    pub targets: Vec<String>,
    pub comment: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortableTermbase {
    pub name: String,
    pub description: Option<String>,
    pub terms: Vec<PortableTerm>,
}
