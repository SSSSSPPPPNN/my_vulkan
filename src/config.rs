use serde_json::Value;

pub fn load_config() -> Config {
    use std::fs;

    let content = fs::read_to_string("config.json").unwrap_or_else(|_| {
        eprintln!("Warning: config.json not found, using defaults");
        "{}".to_string()
    });
    let v: Value = serde_json::from_str(&content).unwrap_or(Value::Null);

    Config {
        validation_layers: v["vulkan"]["validation_layers"].as_bool().unwrap_or(true),
        width: v["window"]["width"].as_u64().unwrap_or(800) as u32,
        height: v["window"]["height"].as_u64().unwrap_or(600) as u32,
        title: v["window"]["title"]
            .as_str()
            .unwrap_or("Vulkan Window")
            .to_string(),
        resizable: v["window"]["resizable"].as_bool().unwrap_or(false),
        rotation_speed: [
            v["vulkan"]["object1"]["rotation_speed"]
                .as_f64()
                .unwrap_or(0.5) as f32,
            v["vulkan"]["object2"]["rotation_speed"]
                .as_f64()
                .unwrap_or(0.5) as f32,
        ],
        vertical_sync: v["vulkan"]["vertical_sync"].as_bool().unwrap_or(true) as bool,
        resize_debounce_ms: v["vulkan"]["resize_debounce_ms"].as_u64().unwrap_or(200) as u32,
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub validation_layers: bool,
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub resizable: bool,
    pub rotation_speed: [f32; 2],
    pub vertical_sync: bool,
    pub resize_debounce_ms: u32,
}
