use chrono::NaiveDateTime;

pub enum Divide {
    Float(f32),
    Timestamp(NaiveDateTime),
    None,
}
