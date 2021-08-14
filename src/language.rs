use num_enum::{IntoPrimitive, TryFromPrimitive};
use r2d2_sqlite::rusqlite::types::{FromSqlResult, Value, ValueRef};
use rusqlite::types::{FromSql, FromSqlError, ToSqlOutput};
use rusqlite::ToSql;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::convert::TryInto;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug, Clone)]
struct DiscrimOutOfRange(i64, &'static str);

impl Display for DiscrimOutOfRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "discriminator {} out of range for type {}",
            self.0, self.1
        )
    }
}

impl Error for DiscrimOutOfRange {}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SerializeDisplay<T>(pub T);

impl<T: Display> Display for SerializeDisplay<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Display> Serialize for SerializeDisplay<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self.0))
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for SerializeDisplay<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer).map(SerializeDisplay)
    }
}

#[derive(
    IntoPrimitive,
    TryFromPrimitive,
    Serialize_repr,
    Deserialize_repr,
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum PartOfSpeech {
    Verb = 1,
    Noun = 2,
    Adjective = 3,
    Adverb = 4,
    Relative = 5,
    Interjection = 6,
    Conjunction = 7,
    Preposition = 8,
    Other = 9,
}

impl FromSql for PartOfSpeech {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let v = value.as_i64()?;
        let err = || FromSqlError::Other(Box::new(DiscrimOutOfRange(v, "PartOfSpeech")));
        Self::try_from_primitive(v.try_into().map_err(|_| err())?).map_err(|_| err())
    }
}

impl ToSql for PartOfSpeech {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(ToSqlOutput::Owned(Value::Integer(*self as u8 as i64)))
    }
}

impl PartOfSpeech {
    /// Used in askama templates
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}

impl Display for PartOfSpeech {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(
    IntoPrimitive,
    TryFromPrimitive,
    Serialize_repr,
    Deserialize_repr,
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum NounClass {
    Class1Um = 1,
    Aba,

    U,
    Oo,

    Class3Um,
    Imi,

    Ili,
    Ama,

    Isi,
    Izi,

    In,
    Izin,

    Ulu,
    Ubu,
    Uku,
}

pub struct NounClassOpt(pub Option<NounClass>);

pub trait NounClassOptExt {
    fn flatten(self) -> Option<NounClass>;
}

impl NounClassOptExt for Option<NounClassOpt> {
    fn flatten(self) -> Option<NounClass> {
        self.and_then(|x| x.0)
    }
}

impl FromSql for NounClassOpt {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let v = value.as_i64()?;

        if v == 255 {
            Ok(NounClassOpt(None))
        } else {
            let err = || FromSqlError::Other(Box::new(DiscrimOutOfRange(v, "NounClass")));
            NounClass::try_from_primitive(v.try_into().map_err(|_| err())?)
                .map_err(|_| err())
                .map(|x| NounClassOpt(Some(x)))
        }
    }
}

impl ToSql for NounClass {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(ToSqlOutput::Owned(Value::Integer(*self as u8 as i64)))
    }
}

pub struct NounClassPrefixes {
    pub singular: &'static str,
    pub plural: Option<&'static str>,
}

impl NounClassPrefixes {
    fn from_singular_plural(singular: &'static str, plural: &'static str) -> Self {
        NounClassPrefixes {
            singular,
            plural: Some(plural),
        }
    }

    fn singular_class(singular: &'static str) -> Self {
        NounClassPrefixes {
            singular,
            plural: None,
        }
    }
}

impl NounClass {
    pub fn to_prefixes(&self) -> NounClassPrefixes {
        use NounClass::*;

        let both = NounClassPrefixes::from_singular_plural;
        let singular = NounClassPrefixes::singular_class;

        match self {
            Class1Um | Aba => both("um", "aba"),
            U | Oo => both("u", "oo"),
            Class3Um | Imi => both("um", "imi"),
            Ili | Ama => both("i(li)", "ama"),
            Isi | Izi => both("isi", "izi"),
            In | Izin => both("i(n)", "i(z)in"),
            Ulu => singular("ulu"),
            Ubu => singular("ubu"),
            Uku => singular("uku"),
        }
    }

    /// Used in askama templates
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}

#[derive(
    IntoPrimitive,
    TryFromPrimitive,
    Serialize_repr,
    Deserialize_repr,
    Copy,
    Clone,
    Debug,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum WordLinkType {
    PluralOrSingular = 1,
    Synonym = 2,
    Antonym = 3,
    Related = 4,
    Confusable = 5,
}

impl WordLinkType {
    fn to_str(&self) -> &'static str {
        match self {
            WordLinkType::PluralOrSingular => "Plural or singular form",
            WordLinkType::Synonym => "Synonym",
            WordLinkType::Antonym => "Antonym",
            WordLinkType::Related => "Related",
            WordLinkType::Confusable => "Confusable",
        }
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}

impl FromSql for WordLinkType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let v = value.as_i64()?;
        let err = || FromSqlError::Other(Box::new(DiscrimOutOfRange(v, "WordLinkType")));
        Self::try_from_primitive(v.try_into().map_err(|_| err())?).map_err(|_| err())
    }
}

impl ToSql for WordLinkType {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(ToSqlOutput::Owned(Value::Integer(*self as u8 as i64)))
    }
}

impl Display for WordLinkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
