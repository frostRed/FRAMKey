use serde_json::{Map, Value, json};

pub(crate) fn payload_summary(value: &Value) -> Value {
    if let Some(text) = value.as_str() {
        if let Some(byte_len) = hex_data_byte_len(text) {
            return json!({
                "encoding": "hex",
                "bytes": byte_len,
                "utf8Preview": decode_hex_utf8_preview(text),
                "preview": preview_string(text, 160),
            });
        }
        return json!({
            "encoding": "text",
            "chars": text.chars().count(),
            "preview": preview_string(text, 160),
        });
    }

    truncate_value(value, 0)
}

pub(crate) fn array_value(params: &Value, index: usize) -> Option<&Value> {
    params.as_array().and_then(|items| items.get(index))
}

pub(crate) fn array_string(params: &Value, index: usize) -> Option<&str> {
    array_value(params, index).and_then(Value::as_str)
}

pub(crate) fn typed_data_param(params: &Value) -> Option<&Value> {
    let items = params.as_array()?;
    items
        .iter()
        .find(|value| {
            value
                .as_str()
                .map(|text| !looks_like_eth_address(text))
                .unwrap_or(true)
        })
        .or_else(|| items.get(1))
}

pub(crate) fn find_first_address(value: &Value) -> Option<String> {
    match value {
        Value::String(text) if looks_like_eth_address(text) => Some(text.clone()),
        Value::Array(items) => items.iter().find_map(find_first_address),
        Value::Object(object) => object.values().find_map(find_first_address),
        _ => None,
    }
}

pub(crate) fn looks_like_eth_address(value: &str) -> bool {
    value.len() == 42
        && value.starts_with("0x")
        && value.as_bytes().iter().skip(2).all(u8::is_ascii_hexdigit)
}

pub(crate) fn parse_json_string_value(value: &Value) -> Value {
    if let Some(text) = value.as_str() {
        serde_json::from_str(text).unwrap_or_else(|_| value.clone())
    } else {
        value.clone()
    }
}

pub(crate) fn truncate_value(value: &Value, depth: usize) -> Value {
    const MAX_DEPTH: usize = 4;
    const MAX_ARRAY_ITEMS: usize = 8;
    const MAX_OBJECT_KEYS: usize = 16;
    const MAX_STRING_CHARS: usize = 220;

    if depth >= MAX_DEPTH {
        return json!({"truncated": "max_depth"});
    }

    match value {
        Value::String(text) => Value::String(preview_string(text, MAX_STRING_CHARS)),
        Value::Array(items) => {
            let mut preview = items
                .iter()
                .take(MAX_ARRAY_ITEMS)
                .map(|item| truncate_value(item, depth + 1))
                .collect::<Vec<_>>();
            if items.len() > MAX_ARRAY_ITEMS {
                preview.push(json!({"truncatedItems": items.len() - MAX_ARRAY_ITEMS}));
            }
            Value::Array(preview)
        }
        Value::Object(object) => {
            let mut preview = Map::new();
            for (key, value) in object.iter().take(MAX_OBJECT_KEYS) {
                preview.insert(key.clone(), truncate_value(value, depth + 1));
            }
            if object.len() > MAX_OBJECT_KEYS {
                preview.insert(
                    "_truncatedKeys".to_owned(),
                    json!(object.len() - MAX_OBJECT_KEYS),
                );
            }
            Value::Object(preview)
        }
        _ => value.clone(),
    }
}

pub(crate) fn preview_string(value: &str, max_chars: usize) -> String {
    let char_count = value.chars().count();
    if char_count <= max_chars {
        return value.to_owned();
    }

    let prefix = value.chars().take(max_chars).collect::<String>();
    format!("{prefix}... ({char_count} chars)")
}

pub(crate) fn hex_data_byte_len(value: &str) -> Option<usize> {
    let hex = value.strip_prefix("0x")?;
    if hex.len() % 2 != 0 || !hex.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    Some(hex.len() / 2)
}

pub(crate) fn decode_hex_utf8_preview(value: &str) -> Option<String> {
    let hex = value.strip_prefix("0x")?;
    let mut bytes = Vec::new();
    for pair in hex.as_bytes().chunks(2).take(96) {
        if pair.len() != 2 {
            return None;
        }
        let high = hex_nibble(pair[0])?;
        let low = hex_nibble(pair[1])?;
        bytes.push((high << 4) | low);
    }

    let text = String::from_utf8(bytes).ok()?;
    if text
        .chars()
        .any(|char| char.is_control() && !char.is_whitespace())
    {
        return None;
    }
    Some(preview_string(&text, 96))
}

pub(crate) fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
