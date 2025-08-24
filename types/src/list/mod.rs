use std::{ fmt::Display};

use serde::{Deserialize, Serialize};

pub mod buckets;
pub mod object;
pub mod prefix;
pub mod v1;
pub mod v2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ListType{
    Version2 = 2,
    Version1 = 1,
}
impl AsRef<str> for ListType {
    fn as_ref(&self) -> &str {
        match self {
            ListType::Version2 => "2",
            ListType::Version1 => "1",
        }
    }
}
impl Display for ListType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
macro_rules! visit_num {
    (
        $(
            fn $visit_fn:ident => type $type:ty
        ),*
    ) => {
        $(
            fn $visit_fn<E>(self, value: $type) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    2 => Ok(ListType::Version2),
                    1 => Ok(ListType::Version1),
                    _ => Err(E::unknown_variant(&value.to_string(), &["1", "2"])),
                }
            }
        )*
    };
}
impl<'de> Deserialize<'de> for ListType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ListType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an integer")
            }

            visit_num! {
                fn visit_u8 => type u8,
                fn visit_u16 => type u16,
                fn visit_u32 => type u32,
                fn visit_u64 => type u64,
                fn visit_i8 => type i8,
                fn visit_i16 => type i16,
                fn visit_i32 => type i32,
                fn visit_i64 => type i64
            }
        }
        deserializer.deserialize_u8(Visitor)
    }
}
impl Serialize for ListType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ListType::Version2 => serializer.serialize_u8(2),
            ListType::Version1 => serializer.serialize_u8(1),
        }
    }
}