use chrono::prelude::*;
use rusqlite::Row;

#[derive(Debug, Clone)]
pub struct Person {
    pub birthday: DateTime<Utc>,
    pub name: String,
    pub user_id: u64,
}

impl Person {
    pub fn age(&self) -> i64 {
        Utc::now().signed_duration_since(self.birthday).num_days() / 365
    }
    pub fn new(birthday: DateTime<Utc>, name: String, user_id: u64) -> Self {
        Self {
            birthday,
            name,
            user_id,
        }
    }
    pub fn get_date_int(&self) -> i64 {
        self.birthday.timestamp()
    }
    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        let name = row.get(2)?;
        let birthday = DateTime::from_timestamp(row.get(1)?, 0).unwrap();
        let uuid = row.get(0)?;

        Ok(Person {
            user_id: uuid,
            name,
            birthday,
        })
    }
}

pub fn to_month(month: u32) -> Month {
    match month {
        1 => Month::January,
        2 => Month::February,
        3 => Month::March,
        4 => Month::April,
        5 => Month::May,
        6 => Month::June,
        7 => Month::July,
        8 => Month::August,
        9 => Month::September,
        10 => Month::October,
        11 => Month::November,
        12 => Month::December,
        _ => Month::January,
    }
}
