use musicmeta::tis::territory::{TisCountryCode, TisRegionCode};
use serde::{Deserialize, Serialize, de};

#[derive(Debug, Clone, Copy)]
pub enum TerritoryCode {
    Region(TisRegionCode),
    Country(TisCountryCode),
}
impl Default for TerritoryCode {
    fn default() -> Self {
        Self::Region(TisRegionCode::default())
    }
}
impl TerritoryCode {
    pub fn code(&self) -> u16 {
        match self {
            TerritoryCode::Region(x) => *x as u16,
            TerritoryCode::Country(x) => *x as u16,
        }
    }
}

impl Serialize for TerritoryCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let num = match self {
            TerritoryCode::Region(code) => *code as u16,
            TerritoryCode::Country(code) => *code as u16,
        };
        serializer.serialize_u16(num)
    }
}

impl<'de> Deserialize<'de> for TerritoryCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let num = u16::deserialize(deserializer)?;
        if let Ok(code) = TisCountryCode::try_from(num) {
            return Ok(TerritoryCode::Country(code));
        }
        if let Ok(code) = TisRegionCode::try_from(num) {
            return Ok(TerritoryCode::Region(code));
        }
        Err(de::Error::custom(format!("invalid TIS code {num}")))
    }
}
