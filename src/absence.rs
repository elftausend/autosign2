use serde_json::Value;

#[derive(Debug)]
pub struct Absence {
    pub id: u64,
    pub start_date: u32,
    pub end_date: u32,
    pub start_time: u32,
    pub end_time: u32,
    pub category: String,
    pub reason_text: String,
    pub is_excused: bool,
}

pub fn read_all_absences(file_path: impl AsRef<str>) -> Option<Vec<Absence>> {
    let value: Value =
        serde_json::from_str(&std::fs::read_to_string(file_path.as_ref()).ok()?).ok()?;
    let absences = value["absences"].as_array()?;
    absences
        .iter()
        .map(|absence| Absence::new(absence))
        .collect()
}

pub fn date_to_format(date: u32) -> String {
    let day = date % 100;
    let month = (date / 100) % 100;
    let year = (date / 10000) % 10000;
    format!("{day}.{month}.{}", year % 100)
}

impl Absence {
    pub fn new(absence: &Value) -> Option<Self> {
        let id = absence["id"].as_u64()?;
        let category = absence["reason"].as_str()?.to_string();
        let reason_text = absence["text"].as_str()?.to_string();
        let start_date = absence["startDate"].as_i64()? as u32;
        let end_date = absence["endDate"].as_i64()? as u32;
        let start_time = absence["startTime"].as_i64()? as u32;
        let end_time = absence["endTime"].as_i64()? as u32;
        let is_excused = absence["isExcused"].as_bool()?;
        Some(Absence {
            id,
            start_date,
            end_date,
            start_time,
            end_time,
            category,
            reason_text,
            is_excused,
        })
    }

    // does not respect breaks, 50 min==1 hour etc
    pub fn hours_absent_estimate(&self) -> u32 {
        (self.end_time - self.start_time) / 100
    }

    pub fn start_date(&self) -> String {
        date_to_format(self.start_date)
    }

    pub fn end_date(&self) -> String {
        date_to_format(self.end_date)
    }
}
