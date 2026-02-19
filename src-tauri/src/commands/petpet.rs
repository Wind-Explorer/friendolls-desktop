use crate::models::dolls::DollDto;
use crate::services::petpet;

#[tauri::command]
pub fn encode_pet_doll_gif_base64(doll: DollDto) -> Result<String, String> {
    petpet::encode_pet_doll_gif_base64(doll)
}
