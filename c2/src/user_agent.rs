// use serde::{Deserialize, Serialize};

// pub struct C2UserAgent {
//     pub name: String,
//     pub version: String,
//     pub os: String,
//     pub hash: String,
// }
//
// impl C2UserAgent {
//     pub fn from(s: String) -> Self {
//         let mut words = s.split_whitespace();
//         let name_version = words.next().unwrap();
//         let mut name_version = name_version.split("/");
//         let name = name_version.next().unwrap();
//         let version = name_version.next().unwrap();
//         let os = words.next().unwrap();
//         let hash = words.next().unwrap();
//         Self {
//             name: name.to_string(),
//             version: version.to_string(),
//             os: os.to_string(),
//             hash: hash.to_string(),
//         }
//     }
// }
//
// impl ToString for C2UserAgent {
//     fn to_string(&self) -> String {
//         format!("{}/{} ({}) {}", self.name, self.version, self.os, self.hash)
//     }
// }

// impl Serialize for C2UserAgent {
//     fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
//         let mut s = String::new();
//         s.push_str(&self.name);
//         s.push_str("/");
//         s.push_str(&self.version);
//         s.push_str(" (");
//         s.push_str(&self.os);
//         s.push_str(")");
//         s.push_str(" ");
//         s.push_str(&self.hash);
//         serializer.serialize_str(&s)
//     }
// }
//
// impl<'de> Deserialize<'de> for C2UserAgent {
//     fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
//         let s = String::deserialize(deserializer)?;
//         Ok(Self::from(s))
//     }
// }
