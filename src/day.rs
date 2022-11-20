use core::fmt;
use core::fmt::Display;
use core::fmt::Debug;
use core::fmt::Write;

use crate::oldtime;
use crate::DateTime;
use crate::NaiveDate;
use crate::NaiveTime;
use crate::TimeZone;

/// Represents the full range of local timestamps in the day
///
///
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Day<Tz>
where
    Tz: TimeZone + Copy + Eq + Display,
{
    date: NaiveDate,
    tz: Tz,
}

impl<Tz> Debug for Day<Tz>
where
    Tz: TimeZone + Copy + Eq + Display,
    {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.date, f)?;
        f.write_char(' ')?;
        self.tz.fmt(f)
    }
}

impl<Tz> Display for Day<Tz>
where
    Tz: TimeZone + Copy + Eq + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}


impl<Tz> Day<Tz>
where
    Tz: TimeZone + Copy + Eq + Display,
{
    ///
    pub fn date(&self) -> NaiveDate {
        self.date
    }

    ///
    pub fn zone(&self) -> Tz {
        self.tz
    }

    ///
    pub fn new(date: NaiveDate, tz: Tz) -> Day<Tz> {
        Day { date, tz }
    }

    ///
    pub fn succ(&self) -> Option<Day<Tz>> {
        Some(Day { date: self.date.succ_opt()?, tz: self.tz })
    }

    ///
    pub fn pred(&self) -> Option<Day<Tz>> {
        Some(Day { date: self.date.pred_opt()?, tz: self.tz })
    }

    ///
    pub fn start(&self) -> DateTime<Tz> {
        // All possible offsets: https://en.wikipedia.org/wiki/List_of_UTC_offsets
        // means a gap of 15 minutes should be reasonable

        // while looping here is less than ideal, in the vast majority of cases
        // the inital start time guess will be valid

        let base = NaiveTime::MIN;
        for multiple in 0..=24 {
            let start_time = base + oldtime::Duration::minutes(multiple * 15);
            match self.tz.from_local_datetime(&self.date.and_time(start_time)) {
                crate::LocalResult::None => continue,
                crate::LocalResult::Single(dt) => return dt,
                crate::LocalResult::Ambiguous(dt1, dt2) => {
                    if dt1.naive_utc() < dt2.naive_utc() {
                        return dt1;
                    } else {
                        return dt2;
                    }
                }
            }
        }

        panic!("Unable to calculate start time for date {} and time zone {}", self.date, self.tz)
    }
}

impl<Tz> From<DateTime<Tz>> for Day<Tz>
where
    Tz: TimeZone + Copy + Eq + Display,
{
    fn from(dt: DateTime<Tz>) -> Self {
        Day { date: dt.date_naive(), tz: dt.timezone() }
    }
}

#[cfg(test)]
mod tests {
    use super::Day;
    use crate::Utc;

    #[test]
    fn test_start_time() {
        assert_eq!(
            Day::from(Utc::now()).start(),
            Utc::now()
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Utc)
                .single()
                .unwrap(),
        );
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
mod serde {
    use core::fmt::{Display};
    use serde::{ser};
    use crate::{Day, TimeZone};

    // Currently no `Deserialize` option as there is no generic way to create a timezone
    // from a string representation of it. This could be added to the `TimeZone` trait in future

    impl<Tz> ser::Serialize for Day<Tz> where
    Tz: TimeZone + Copy + Eq + Display,{
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            serializer.serialize_str(&self.to_string())
        }
    }
}
