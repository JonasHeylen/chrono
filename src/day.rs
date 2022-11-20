use core::cmp::Ordering;
use core::convert::TryFrom;
use core::fmt;
use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Write;
use core::ops::Add;
use core::ops::Sub;

use crate::oldtime;
use crate::DateTime;
use crate::Days;
use crate::NaiveDate;
use crate::NaiveTime;
use crate::TimeZone;
use oldtime::Duration;

/// Represents the full range of local timestamps in the day
///
///
#[derive(Clone, Copy)]
pub struct Day<Tz>
where
    Tz: TimeZone + Copy + Display,
{
    date: NaiveDate,
    tz: Tz,
}

impl<Tz> Debug for Day<Tz>
where
    Tz: TimeZone + Copy + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.date, f)?;
        f.write_char(' ')?;
        self.tz.fmt(f)
    }
}

impl<Tz> Display for Day<Tz>
where
    Tz: TimeZone + Copy + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl<Tz> PartialOrd for Day<Tz>
where
    Tz: TimeZone + Copy + Display,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.date.partial_cmp(&other.date)
    }
}

impl<Tz> Ord for Day<Tz>
where
    Tz: TimeZone + Copy + Display,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.date.cmp(&other.date)
    }
}

impl<Tz> PartialEq for Day<Tz>
where
    Tz: TimeZone + Copy + Display,
{
    fn eq(&self, other: &Self) -> bool {
        self.date.eq(&other.date)
    }
}

impl<Tz> Eq for Day<Tz> where Tz: TimeZone + Copy + Display {}

impl<Tz> Day<Tz>
where
    Tz: TimeZone + Copy + Display,
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

    ///
    pub fn checked_add_days(self, days: Days) -> Option<Self> {
        if days.0 == 0 {
            return Some(self);
        }

        i64::try_from(days.0).ok().and_then(|d| self.diff_days(d))
    }

    ///
    pub fn checked_sub_days(self, days: Days) -> Option<Self> {
        if days.0 == 0 {
            return Some(self);
        }

        i64::try_from(days.0).ok().and_then(|d| self.diff_days(-d))
    }

    fn diff_days(self, days: i64) -> Option<Self> {
        let date = self.date.checked_add_signed(Duration::days(days))?;
        Some(Day { date, ..self })
    }
}

impl<Tz> Add<Days> for Day<Tz>
where
    Tz: TimeZone + Copy + Display,
{
    type Output = Day<Tz>;

    fn add(self, days: Days) -> Self::Output {
        self.checked_add_days(days).unwrap()
    }
}

impl<Tz> Sub<Days> for Day<Tz>
where
    Tz: TimeZone + Copy + Display,
{
    type Output = Day<Tz>;

    fn sub(self, days: Days) -> Self::Output {
        self.checked_sub_days(days).unwrap()
    }
}

impl<Tz> From<DateTime<Tz>> for Day<Tz>
where
    Tz: TimeZone + Copy + Display,
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
    use crate::{Day, TimeZone};
    use core::fmt::Display;
    use serde::ser;

    // Currently no `Deserialize` option as there is no generic way to create a timezone
    // from a string representation of it. This could be added to the `TimeZone` trait in future

    impl<Tz> ser::Serialize for Day<Tz>
    where
        Tz: TimeZone + Copy + Display,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            let display = self.to_string();
            serializer.serialize_str(&display)
        }
    }
}
