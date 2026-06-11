use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Party {
    #[serde(rename = "#PartyRecordId")]
    pub id: i64,
    #[serde(rename = "EmailAddress")]
    pub email: Option<String>,
    #[serde(rename = "ISNI")]
    pub isni: Option<String>,
    #[serde(rename = "CisacID")]
    pub cisac_id: Option<String>,
    #[serde(rename = "DPID")]
    pub dpid: Option<String>,
    #[serde(rename = "IpiNameNumber")]
    pub ipi: Option<i64>,
    #[serde(rename = "ContactName")]
    pub contact_name: Option<String>,
    #[serde(rename = "FullName")]
    pub full_name: String,
    #[serde(rename = "KeyName")]
    pub last_name: Option<String>,
    #[serde(rename = "Prefix")]
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
    #[serde(rename = "NominalDuration")]
    pub duration_ms: f64,
    #[serde(rename = "ISWC")]
    pub iswc: Option<String>,
    #[serde(rename = "HasRightShareInDispute")]
    pub in_dispute: bool,
    #[serde(rename = "AlternativeMusicalWorkIdForUsStatutoryReversion")]
    pub alt_id: Option<i64>,
    #[serde(rename = "IsArrangementOfTraditionalWork")]
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
