use serde::{Deserialize, Deserializer, Serialize, de};

fn named_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let res = String::deserialize(deserializer)?;
    match res.as_str() {
        "TRUE" => Ok(true),
        "FALSE" => Ok(false),
        _ => return Err(de::Error::custom("invalid bool")),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Party {
    #[serde(rename = "#PartyRecordId")]
    pub id: i64,
    #[serde(rename = "EmailAddress", default)]
    pub email: Option<String>,
    #[serde(rename = "ISNI", default)]
    pub isni: Option<String>,
    #[serde(rename = "CisacID", alias = "CisacSocietyId", default)]
    pub cisac_id: Option<String>,
    #[serde(rename = "DPID", default)]
    pub dpid: Option<String>,
    #[serde(rename = "IpiNameNumber", default)]
    pub ipi: Option<i64>,
    #[serde(rename = "ContactName", default)]
    pub contact_name: Option<String>,
    #[serde(rename = "FullName")]
    pub full_name: String,
    #[serde(rename = "KeyName", default)]
    pub last_name: Option<String>,
    #[serde(rename = "Prefix", alias = "NamesBeforeKeyName", default)]
    pub first_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Release {
    #[serde(rename = "#ReleaseRecordId")]
    pub id: i64,
    #[serde(rename = "ReleaseTitle")]
    pub title: String,
    #[serde(rename = "DisplayArtistName")]
    pub artist_name: String,
    #[serde(rename = "LabelName")]
    pub label_name: String,
    #[serde(rename = "DistributorName")]
    pub distro_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Work {
    #[serde(rename = "#MusicalWorkRecordId")]
    pub id: String,
    #[serde(rename = "MusicalWorkTitle")]
    pub title: String,
    #[serde(rename = "NominalDuration", default)]
    pub duration_ms: Option<f64>,
    #[serde(rename = "ISWC")]
    pub iswc: Option<String>,
    #[serde(rename = "HasRightShareInDispute", deserialize_with = "named_bool")]
    pub in_dispute: bool,
    #[serde(rename = "AlternativeMusicalWorkIdForUsStatutoryReversion")]
    pub alt_id: Option<i64>,
    #[serde(
        rename = "IsArrangementOfTraditionalWork",
        deserialize_with = "named_bool"
    )]
    pub is_arrangement: bool,
    #[serde(rename = "TerritoryOfPublicDomain")]
    pub territory: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Share {
    #[serde(rename = "#MusicalWorkRightShareRecordId")]
    pub id: String,
    #[serde(rename = "MusicalWorkRecordId")]
    pub work_id: String,
    #[serde(rename = "PartyRecordId")]
    pub party_id: i64,
    #[serde(rename = "PartyRole")]
    pub role: String,
    #[serde(rename = "RightShareType")]
    pub share_type: String,
    #[serde(rename = "RightsType")]
    pub rights_type: String,
    #[serde(rename = "RightSharePercentage")]
    pub share: Option<f64>,
    #[serde(rename = "TerritoryCode")]
    pub territory_code: String,
    #[serde(rename = "PrecedingMusicalWorkRightShareRecordId")]
    pub preceding_id: Option<String>,
}
