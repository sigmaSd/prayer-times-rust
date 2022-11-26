use crate::float_time_to_time24;

impl crate::PrayerTimes {
    pub fn get_prayer_times_easy24(
        &mut self,
        year: usize,
        month: usize,
        day: usize,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> EasyTimes24 {
        let times = self.get_prayer_times(year, month, day, latitude, longitude, timezone);
        EasyTimes24 {
            fajr: float_time_to_time24(times[0]),
            sunrise: float_time_to_time24(times[1]),
            dhuhr: float_time_to_time24(times[2]),
            asr: float_time_to_time24(times[3]),
            sunset: float_time_to_time24(times[4]),
            maghrib: float_time_to_time24(times[5]),
            isha: float_time_to_time24(times[6]),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct EasyTimes24 {
    pub fajr: String,
    pub sunrise: String,
    pub dhuhr: String,
    pub asr: String,
    pub sunset: String,
    pub maghrib: String,
    pub isha: String,
}

#[cfg(test)]
mod tests {
    use crate::{AdjustingMethod, CalculationMethod, JuristicMethod, PrayerTimes};

    #[test]
    fn test_easy() {
        let mut pt = PrayerTimes::new(
            CalculationMethod::MWL,
            JuristicMethod::default(),
            AdjustingMethod::default(),
            Default::default(),
        );
        let times = pt.get_prayer_times_easy24(2022, 11, 26, 36., 10., 1.);
        dbg!(&times);
    }
}
