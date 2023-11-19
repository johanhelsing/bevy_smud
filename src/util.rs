use bevy::utils::Uuid;

pub fn generate_shader_id() -> String {
    Uuid::new_v4().to_string().replace('-', "_")
}
