use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct LogEntry {
    fields: Fields,
}

#[derive(Debug, Deserialize)]
struct Fields {
    message: String,
    tip: Option<String>,
}

pub fn extract_tip_changed(line: &str) -> Option<u64> {
    let json_part = line.find('{').map(|i| &line[i..])?;
    let entry: LogEntry = serde_json::from_str(json_part).ok()?;
    if entry.fields.message == "tip_changed" {
        if let Some(tip) = entry.fields.tip {
            if let Some(slot_str) = tip.split('.').next() {
                return slot_str.parse::<u64>().ok();
            }
        }
    }
    None
}
