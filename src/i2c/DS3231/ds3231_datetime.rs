use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Метка времени
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Ds3231Datetime {
    /// Год (0-99).
    ///
    /// Если необходимо задать из четырехзначного года:
    ///
    /// ```rust
    /// let year: u8 = (now.year() % 100) as u8;
    /// ```
    pub year: u8,

    /// Месяц (1-12).
    pub month: u8,

    /// День (1-31).
    pub day: u8,

    /// Час (0-23).
    pub hour: u8,

    /// Минута (0-59).
    pub minute: u8,

    /// Секунда (0-59).
    pub second: u8,
}
impl Display for Ds3231Datetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Time: {:02}-{:02}-{:02}T{:02}:{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second,
        )
    }
}

#[cfg(feature = "time")]
impl Ds3231Datetime {
    /// Возвращает текущее время из библиотеки `time`.
    pub fn now_from_crate_time() -> Self {
        let now = time::OffsetDateTime::now_utc();

        let year: u8 = (now.year() % 100) as u8;

        Self {
            year,
            month: now.month() as u8,
            day: now.day(),
            hour: now.hour(),
            minute: now.minute(),
            second: now.second(),
        }
    }

    /// Преобразует метку времени в структуру `time::OffsetDateTime`.
    pub fn into_crate_time(&self) -> Result<time::OffsetDateTime, time::Error> {
        use time::Time;

        let year = self.year as i32 + 2000;
        let month: time::Month = self.month.try_into()?;
        let day: u8 = self.day;
        let date = time::Date::from_calendar_date(year, month, day)?;

        let time: time::Time = Time::from_hms(self.hour, self.minute, self.second)?;

        Ok(time::OffsetDateTime::new_utc(date, time))
    }
}
