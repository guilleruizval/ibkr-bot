use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use eyre::{Result, eyre};

use crate::constants::ASX_TZ;

#[derive(Debug, Clone)]
pub struct ParsedTradingHours {
    tz: Tz,
    inner: Vec<TradingHoursEntry>,
}

#[derive(Debug, Clone)]
enum TradingHoursEntry {
    Open(Hours),
    Closed,
}

#[derive(Debug, Clone)]
pub struct Hours {
    pub open_local: DateTime<Tz>,
    #[allow(unused)]
    pub close_local: DateTime<Tz>,
}

impl ParsedTradingHours {
    pub fn next_session(&self) -> Option<Hours> {
        let now_utc = Utc::now();
        let now_local = now_utc.with_timezone(&self.tz);
        self.inner
            .iter()
            .filter_map(|entry| match entry {
                TradingHoursEntry::Open(hours) => Some(hours.clone()),
                TradingHoursEntry::Closed => None,
            })
            .filter(|hours| hours.open_local > now_local)
            .min_by_key(|hours| hours.open_local)
    }
    pub fn parse(tz: Tz, trading_hours: Vec<String>) -> Result<Self> {
        if tz.name() != ASX_TZ {
            return Err(eyre!("ASX timezone mismatch: {}", tz.name()));
        }
        let inner = trading_hours
            .into_iter()
            .map(|string| {
                let (date_str, times) = string.split_once(':').unwrap();

                let year = date_str[0..4].parse::<i32>().unwrap();
                let month = date_str[4..6].parse::<u32>().unwrap();
                let day = date_str[6..8].parse::<u32>().unwrap();
                let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();

                if times == "CLOSED" {
                    TradingHoursEntry::Closed
                } else {
                    // Open at 10AM, close at 4PM (Australian time)
                    let open_time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
                    let close_time = NaiveTime::from_hms_opt(16, 0, 0).unwrap();

                    let open_local = tz
                        .from_local_datetime(&date.and_time(open_time))
                        .single()
                        .unwrap();
                    let close_local = tz
                        .from_local_datetime(&date.and_time(close_time))
                        .single()
                        .unwrap();

                    TradingHoursEntry::Open(Hours {
                        open_local,
                        close_local,
                    })
                }
            })
            .collect::<Vec<_>>();

        Ok(Self { tz, inner })
    }
}
