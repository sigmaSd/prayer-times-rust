/*-------------------------- In the name of God ----------------------------*\

    prayer_times (rust)
    Islamic prayer times calculator library
    Based on libprayertimes 1.0 Cpp library
    Based on PrayTimes 1.1 JavaScript library

----------------------------- Copyright Block --------------------------------

Copyright (C) 2007-2010 PrayTimes.org

Developed By: Nbiba Bedis <bedis at nbiba at gmail dot com>
Based on Cpp Code By: Mohammad Ebrahim Mohammadi Panah <ebrahim at mohammadi dot ir>
Based on a JavaScript Code By: Hamid Zarrabi-Zadeh

License: GNU GPL v3.0

TERMS OF USE:
    Permission is granted to use this code, with or
    without modification, in any website or application
    provided that credit is given to the original work
    with a link back to PrayTimes.org.

This program is distributed in the hope that it will
be useful, but WITHOUT ANY WARRANTY.

PLEASE DO NOT REMOVE THIS COPYRIGHT BLOCK.

------------------------------------------------------------------------------

User's Manual:
http://praytimes.org/manual

Calculating Formulas:
http://praytimes.org/calculation

\*--------------------------------------------------------------------------*/

use std::f64::consts::PI;

pub mod easy;

const INVALID_TIME: &str = "-----";

/* convert float hours to 24h format */
pub fn float_time_to_time24(time: f64) -> String {
    if f64::is_nan(time) {
        return INVALID_TIME.into();
    }
    let (hours, minutes) = get_float_time_parts(time);
    format!("{:02}:{:02}", hours, minutes)
}

#[derive(Default)]
pub struct PrayerTimes {
    method_params: [MethodConfig; CalculationMethod::CalculationMethodsCount as usize],

    calc_method: CalculationMethod,    // caculation method
    asr_juristic: JuristicMethod,      // Juristic method for Asr
    adjust_high_lats: AdjustingMethod, // adjusting method for higher latitudes
    dhuhr_minutes: f64,                // minutes after mid-day for Dhuhr

    latitude: f64,
    longitude: f64,
    timezone: f64,
    julian_date: f64,
}

impl PrayerTimes {
    pub fn new(
        calc_method: CalculationMethod,
        asr_juristic: JuristicMethod,
        adjust_high_lats: AdjustingMethod,
        dhuhr_minutes: f64,
    ) -> Self {
        let mut this = Self {
            calc_method,
            asr_juristic,
            adjust_high_lats,
            dhuhr_minutes,
            ..Default::default()
        };
        use CalculationMethod::*;

        this.method_params[Jafari as usize] = MethodConfig {
            fajr_angle: 16.0,
            maghrib_is_minutes: false,
            maghrib_value: 4.0,
            isha_is_minutes: false,
            isha_value: 14.0,
        }; // Jafari
        this.method_params[Karachi as usize] = MethodConfig {
            fajr_angle: 18.0,
            maghrib_is_minutes: true,
            maghrib_value: 0.0,
            isha_is_minutes: false,
            isha_value: 18.0,
        }; // Karachi
        this.method_params[ISNA as usize] = MethodConfig {
            fajr_angle: 15.0,
            maghrib_is_minutes: true,
            maghrib_value: 0.0,
            isha_is_minutes: false,
            isha_value: 15.0,
        }; // ISNA
        this.method_params[MWL as usize] = MethodConfig {
            fajr_angle: 18.0,
            maghrib_is_minutes: true,
            maghrib_value: 0.0,
            isha_is_minutes: false,
            isha_value: 17.0,
        }; // MWL
        this.method_params[Makkah as usize] = MethodConfig {
            fajr_angle: 19.0,
            maghrib_is_minutes: true,
            maghrib_value: 0.0,
            isha_is_minutes: true,
            isha_value: 90.0,
        }; // Makkah
        this.method_params[Egypt as usize] = MethodConfig {
            fajr_angle: 19.5,
            maghrib_is_minutes: true,
            maghrib_value: 0.0,
            isha_is_minutes: false,
            isha_value: 17.5,
        }; // Egypt
        this.method_params[Custom as usize] = MethodConfig {
            fajr_angle: 18.0,
            maghrib_is_minutes: true,
            maghrib_value: 0.0,
            isha_is_minutes: false,
            isha_value: 17.0,
        }; // Custom
        this
    }
    pub fn get_prayer_times(
        &mut self,
        year: usize,
        month: usize,
        day: usize,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> [f64; TimeID::TimesCount as usize] {
        self.latitude = latitude;
        self.longitude = longitude;
        self.timezone = timezone;
        self.julian_date = get_julian_date(year, month, day) - longitude / (15. * 24.);
        self.compute_day_times()
    }
    fn compute_day_times(&self) -> [f64; TimeID::TimesCount as usize] {
        const NUM_ITERATIONS: usize = 1; // number of iterations needed to compute times
        let mut times = [5., 6., 12., 13., 18., 18., 18.]; // default times

        for _ in 0..NUM_ITERATIONS {
            self.compute_times(&mut times);
        }

        self.adjust_times(&mut times);

        times
    }

    /* adjust times in a prayer time array */
    fn adjust_times(&self, times: &mut [f64; TimeID::TimesCount as usize]) {
        use TimeID::*;
        for time in times.iter_mut().take(TimeID::TimesCount as usize) {
            *time += self.timezone - self.longitude / 15.0;
        }
        times[Dhuhr as usize] += self.dhuhr_minutes / 60.0; // Dhuhr
        if self.method_params[self.calc_method as usize].maghrib_is_minutes {
            // Maghrib
            times[Maghrib as usize] = times[Sunset as usize]
                + self.method_params[self.calc_method as usize].maghrib_value / 60.0;
        }
        if self.method_params[self.calc_method as usize].isha_is_minutes {
            // Isha
            times[Isha as usize] = times[Maghrib as usize]
                + self.method_params[self.calc_method as usize].isha_value / 60.0;
        }

        if self.adjust_high_lats != AdjustingMethod::None {
            self.adjust_high_lat_times(times);
        }
    }

    /* adjust Fajr, Isha and Maghrib for locations in higher latitudes */
    fn adjust_high_lat_times(&self, times: &mut [f64; TimeID::TimesCount as usize]) {
        use TimeID::*;
        let night_time = time_diff(times[Sunset as usize], times[Sunrise as usize]); // sunset to sunrise

        // Adjust Fajr
        let fajr_diff = self
            .night_portion(self.method_params[self.calc_method as usize].fajr_angle)
            * night_time;
        if f64::is_nan(times[Fajr as usize])
            || time_diff(times[Fajr as usize], times[Sunrise as usize]) > fajr_diff
        {
            times[Fajr as usize] = times[Sunrise as usize] - fajr_diff;
        }

        // Adjust Isha
        let isha_angle = if self.method_params[self.calc_method as usize].isha_is_minutes {
            18.0
        } else {
            self.method_params[self.calc_method as usize].isha_value
        };
        let isha_diff = self.night_portion(isha_angle) * night_time;
        if f64::is_nan(times[Isha as usize])
            || time_diff(times[Sunset as usize], times[Isha as usize]) > isha_diff
        {
            times[Isha as usize] = times[Sunset as usize] + isha_diff;
        }

        // Adjust Maghrib
        let maghrib_angle = if self.method_params[self.calc_method as usize].maghrib_is_minutes {
            4.0
        } else {
            self.method_params[self.calc_method as usize].maghrib_value
        };
        let maghrib_diff = self.night_portion(maghrib_angle) * night_time;
        if f64::is_nan(times[Maghrib as usize])
            || time_diff(times[Sunset as usize], times[Maghrib as usize]) > maghrib_diff
        {
            times[Maghrib as usize] = times[Sunset as usize] + maghrib_diff;
        }
    }

    /* ---------------------- Compute Prayer Times ----------------------- */

    // array parameters must be at least of size TimesCount

    /* compute prayer times at given julian date */
    fn compute_times(&self, times: &mut [f64; TimeID::TimesCount as usize]) {
        day_portion(times);

        use TimeID::*;
        times[Fajr as usize] = self.compute_time(
            180.0 - self.method_params[self.calc_method as usize].fajr_angle,
            times[Fajr as usize],
        );
        times[Sunrise as usize] = self.compute_time(180.0 - 0.833, times[Sunrise as usize]);
        times[Dhuhr as usize] = self.compute_mid_day(times[Dhuhr as usize]);
        times[Asr as usize] = self.compute_asr(1 + self.asr_juristic as usize, times[Asr as usize]);
        times[Sunset as usize] = self.compute_time(0.833, times[Sunset as usize]);
        times[Maghrib as usize] = self.compute_time(
            self.method_params[self.calc_method as usize].maghrib_value,
            times[Maghrib as usize],
        );
        times[Isha as usize] = self.compute_time(
            self.method_params[self.calc_method as usize].isha_value,
            times[Isha as usize],
        );
    }

    /* compute the time of Asr */
    fn compute_asr(&self, step: usize, t: f64) -> f64 // Shafii: step=1, Hanafi: step=2
    {
        let d = sun_declination(self.julian_date + t);
        let g = -darccot(step as f64 + dtan(f64::abs(self.latitude - d)));
        self.compute_time(g, t)
    }

    /* compute time for a given angle G */
    fn compute_time(&self, g: f64, t: f64) -> f64 {
        let d = sun_declination(self.julian_date + t);
        let z = self.compute_mid_day(t);
        let v = 1.0 / 15.0
            * darccos((-dsin(g) - dsin(d) * dsin(self.latitude)) / (dcos(d) * dcos(self.latitude)));
        z + (if g > 90.0 { -v } else { v })
    }
    /* compute mid-day (Dhuhr, Zawal) time */
    fn compute_mid_day(&self, _t: f64) -> f64 {
        let t = equation_of_time(self.julian_date + _t);
        fix_hour(12. - t)
    }
    /* the night portion used for adjusting times in higher latitudes */
    fn night_portion(&self, angle: f64) -> f64 {
        match self.adjust_high_lats {
            AdjustingMethod::AngleBased => angle / 60.0,
            AdjustingMethod::MidNight => 1.0 / 2.0,
            AdjustingMethod::OneSeventh => 1.0 / 7.0,
            _ => {
                // In original library nothing was returned
                // Maybe I should throw an exception
                // It must be impossible to reach here
                0.
            }
        }
    }
}

/* compute equation of time */
fn equation_of_time(jd: f64) -> f64 {
    sun_position(jd).1
}

fn get_julian_date(mut year: usize, mut month: usize, day: usize) -> f64 {
    if month <= 2 {
        year -= 1;
        month += 12;
    }

    let a = (year as f64 / 100.).floor();
    let b = 2. - a + (a / 4.).floor();

    (365.25 * (year + 4716) as f64).floor()
        + (30.6001 * (month + 1) as f64).floor()
        + day as f64
        + b
        - 1524.5
}

/* compute declination angle of sun */
fn sun_declination(jd: f64) -> f64 {
    sun_position(jd).0
}

/* compute declination angle of sun and equation of time */
fn sun_position(jd: f64) -> (f64, f64) {
    let d = jd - 2451545.0;
    let g = fix_angle(357.529 + 0.98560028 * d);
    let q = fix_angle(280.459 + 0.98564736 * d);
    let l = fix_angle(q + 1.915 * dsin(g) + 0.020 * dsin(2. * g));

    // double r = 1.00014 - 0.01671 * dcos(g) - 0.00014 * dcos(2 * g);
    let e = 23.439 - 0.00000036 * d;

    let dd = darcsin(dsin(e) * dsin(l));
    let ra = darctan2(dcos(e) * dsin(l), dcos(l)) / 15.0;
    let ra = fix_hour(ra);
    let eq_t = q / 15.0 - ra;

    (dd, eq_t)
}

/* ---------------------- Trigonometric Functions ----------------------- */

/* degree sin */
fn dsin(d: f64) -> f64 {
    deg2rad(d).sin()
}

/* degree cos */
fn dcos(d: f64) -> f64 {
    deg2rad(d).cos()
}

/* degree tan */
fn dtan(d: f64) -> f64 {
    deg2rad(d).tan()
}

/* degree arcsin */
fn darcsin(x: f64) -> f64 {
    rad2deg(x.asin())
}

/* degree arccos */
fn darccos(x: f64) -> f64 {
    rad2deg(x.acos())
}

/* degree arctan */
fn _darctan(x: f64) -> f64 {
    rad2deg(x.atan())
}

/* degree arctan2 */
fn darctan2(y: f64, x: f64) -> f64 {
    rad2deg(y.atan2(x))
}

/* degree arccot */
fn darccot(x: f64) -> f64 {
    rad2deg((1.0 / x).atan())
}

/* degree to radian */
fn deg2rad(d: f64) -> f64 {
    d * PI / 180.0
}

/* radian to degree */
fn rad2deg(r: f64) -> f64 {
    r * 180.0 / PI
}

/* range reduce angle in degrees. */
fn fix_angle(mut a: f64) -> f64 {
    a = a - 360.0 * (a / 360.0).floor();
    if a < 0.0 {
        a + 360.0
    } else {
        a
    }
}

/* range reduce hours to 0..23 */
fn fix_hour(mut a: f64) -> f64 {
    a = a - 24.0 * (a / 24.0).floor();
    if a < 0.0 {
        a + 24.0
    } else {
        a
    }
}

/* convert hours to day portions  */
fn day_portion(times: &mut [f64; TimeID::TimesCount as usize]) {
    for time in times.iter_mut().take(TimeID::TimesCount as usize) {
        *time /= 24.0;
    }
}

/* get hours and minutes parts of a float time */
fn get_float_time_parts(mut time: f64) -> (usize, usize) {
    time = fix_hour(time + 0.5 / 60.); // add 0.5 minutes to round
    let hours = f64::floor(time) as usize;
    let minutes = f64::floor((time - hours as f64) * 60.) as usize;
    (hours, minutes)
}

/* ---------------------- Misc Functions ----------------------- */

/* compute the difference between two times  */
fn time_diff(time1: f64, time2: f64) -> f64 {
    fix_hour(time2 - time1)
}

// Calculation Methods
#[derive(Default, PartialEq, Eq, Hash, Clone, Copy)]
pub enum CalculationMethod {
    Jafari,  // Ithna Ashari
    Karachi, // University of Islamic Sciences, Karachi
    ISNA,    // Islamic Society of North America (ISNA)
    #[default]
    MWL, // Muslim World League (MWL)
    Makkah,  // Umm al-Qura, Makkah
    Egypt,   // Egyptian General Authority of Survey
    Custom,  // Custom Setting

    CalculationMethodsCount,
}
// Juristic Methods
#[derive(Default, Clone, Copy)]
pub enum JuristicMethod {
    #[default]
    Shafii = 0, // Shafii (standard)
    Hanafi, // Hanafi
}
// Adjusting Methods for Higher Latitudes
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum AdjustingMethod {
    None, // No adjustment
    #[default]
    MidNight, // middle of night
    OneSeventh, // 1/7th of night
    AngleBased, // angle/60th of night
}

#[derive(Default)]
struct MethodConfig {
    fajr_angle: f64,
    maghrib_is_minutes: bool,
    maghrib_value: f64,
    isha_is_minutes: bool,
    isha_value: f64,
}

pub enum TimeID {
    Fajr = 0,
    Sunrise,
    Dhuhr,
    Asr,
    Sunset,
    Maghrib,
    Isha,

    TimesCount,
}

#[cfg(test)]
mod tests {
    use crate::{
        float_time_to_time24, AdjustingMethod, CalculationMethod, JuristicMethod, PrayerTimes,
        TimeID,
    };

    #[test]
    fn test_times() {
        let mut pt = PrayerTimes::new(
            CalculationMethod::default(),
            JuristicMethod::default(),
            AdjustingMethod::default(),
            Default::default(),
        );
        let times = pt.get_prayer_times(2022, 11, 27, 36., 10., 1.);
        for i in 0..TimeID::TimesCount as usize {
            println!("{} : {}\n", TIME_NAME[i], float_time_to_time24(times[i]));
        }
    }

    const TIME_NAME: [&'static str; 7] = [
        "Fajr", "Sunrise", "Dhuhr", "Asr", "Sunset", "Maghrib", "Isha",
    ];
}
