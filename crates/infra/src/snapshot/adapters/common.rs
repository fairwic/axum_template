use serde_json::Value;

pub fn pick_string(payload: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|k| {
        payload.get(*k).and_then(|v| {
            if let Some(s) = v.as_str() {
                Some(s.to_string())
            } else if let Some(i) = v.as_i64() {
                Some(i.to_string())
            } else {
                v.as_u64().map(|u| u.to_string())
            }
        })
    })
}

pub fn pick_i64(payload: &Value, keys: &[&str]) -> Option<i64> {
    keys.iter().find_map(|k| {
        payload.get(*k).and_then(|v| {
            if let Some(i) = v.as_i64() {
                Some(i)
            } else if let Some(u) = v.as_u64() {
                i64::try_from(u).ok()
            } else if let Some(f) = v.as_f64() {
                Some(f as i64)
            } else {
                v.as_str().and_then(|s| s.parse::<i64>().ok())
            }
        })
    })
}

pub fn pick_f64(payload: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|k| {
        payload.get(*k).and_then(|v| {
            if let Some(f) = v.as_f64() {
                Some(f)
            } else if let Some(i) = v.as_i64() {
                Some(i as f64)
            } else {
                v.as_str().and_then(|s| s.parse::<f64>().ok())
            }
        })
    })
}

pub fn pick_images(payload: &Value) -> Vec<String> {
    payload
        .get("images")
        .or_else(|| payload.get("image_urls"))
        .or_else(|| payload.get("gallery_urls"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub fn pick_first_array_item_string(payload: &Value, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(|v| {
            if let Some(i) = v.as_i64() {
                Some(i.to_string())
            } else if let Some(u) = v.as_u64() {
                Some(u.to_string())
            } else {
                v.as_str().map(ToString::to_string)
            }
        })
}
