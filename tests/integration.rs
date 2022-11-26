#[cfg(test)]
mod tests {
    use nanoserde::{DeJson, SerJson};
    use proptest::prelude::*;

    use std::process::Command;

    use prayer_times::{
        easy::EasyTimes24, AdjustingMethod, CalculationMethod, JuristicMethod, PrayerTimes,
    };

    proptest! {
    #[test]
    fn validate_to_js(year in 1000..3000, month in 1..12, day in 1..31 ) {
        let year = year as usize;
        let month = month as usize;
        let day = day as usize;
        let date = Date { year, month, day, lat: 36.,lon:10.,tz:1.};
        let rust_times = get_rust_times(date);
        let js_times = get_js_times(date);
        prop_assert_eq!(rust_times, js_times);
    }
        }
    #[test]
    fn validate_to_js_simple() {
        let year = 1911;
        let month = 3;
        let day = 11;
        let date = Date {
            year,
            month,
            day,
            lat: 36.,
            lon: 10.,
            tz: 1.,
        };
        let rust_times = get_rust_times(date);
        let js_times = get_js_times(date);
        assert_eq!(rust_times, js_times);
    }

    #[derive(proptest_derive::Arbitrary, Copy, Clone, SerJson, DeJson, Debug)]
    struct Date {
        year: usize,
        month: usize,
        day: usize,
        lat: f64,
        lon: f64,
        tz: f64,
    }

    fn get_rust_times(date: Date) -> EasyTimes24 {
        let mut pt = PrayerTimes::new(
            CalculationMethod::default(),
            JuristicMethod::default(),
            AdjustingMethod::default(),
            Default::default(),
        );
        pt.get_prayer_times_easy24(date.year, date.month, date.day, date.lat, date.lon, date.tz)
    }
    fn get_js_times(date: Date) -> EasyTimes24 {
        #[derive(Debug, DeJson, SerJson)]
        pub struct EasyTimes24Json {
            pub fajr: String,
            pub sunrise: String,
            pub dhuhr: String,
            pub asr: String,
            pub sunset: String,
            pub maghrib: String,
            pub isha: String,
        }
        impl Into<EasyTimes24> for EasyTimes24Json {
            fn into(self) -> EasyTimes24 {
                EasyTimes24 {
                    fajr: self.fajr,
                    sunrise: self.sunrise,
                    dhuhr: self.dhuhr,
                    asr: self.asr,
                    sunset: self.sunset,
                    maghrib: self.maghrib,
                    isha: self.isha,
                }
            }
        }
        let out = Command::new("deno")
            .stderr(std::process::Stdio::inherit())
            .args(&["run", "js/example.ts", &date.serialize_json()])
            .output()
            .unwrap();
        let out: EasyTimes24Json =
            DeJson::deserialize_json(&String::from_utf8(out.stdout).unwrap()).unwrap();
        out.into()
    }
}
