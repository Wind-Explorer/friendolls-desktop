use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct DollColorSchemeDto {
    pub outline: String,
    pub body: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct DollConfigurationDto {
    pub color_scheme: DollColorSchemeDto,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct CreateDollDto {
    pub name: String,
    pub configuration: Option<DollConfigurationDto>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDollDto {
    pub name: Option<String>,
    pub configuration: Option<DollConfigurationDto>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct DollDto {
    pub id: String,
    pub name: String,
    pub configuration: DollConfigurationDto,
    pub created_at: String,
    pub updated_at: String,
}
