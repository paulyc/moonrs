extern crate hifitime;
pub use hifitime::*;

pub mod moon {

    use hifitime::Epoch;

    extern crate vsop87;
    use vsop87::vsop87a;
    use vsop87::RectangularCoordinates;

    pub trait ConvertibleToSpherical {
        fn longitude(&self) -> f64;
        fn magnitude(&self) -> f64;
    }

    impl ConvertibleToSpherical for RectangularCoordinates {
        fn longitude(&self) -> f64 {
            (&self.y / &self.x).atan()
        }
        fn magnitude(&self) -> f64 {
            (&self.x * &self.x + &self.y * &self.y + &self.z * &self.z).sqrt()
        }
    }

    #[derive(Debug)]
    enum PhaseDiffState {
        Initial,
        First,
        Positive,
        Negative,
    }

    pub trait MoonFinder {
        fn mut_last_moon(self: &mut Self);
        fn mut_next_moon(self: &mut Self);
    }

    impl MoonFinder for Epoch {
        fn mut_last_moon(self: &mut Self) {
            self.mut_next_moon();
            self.mut_sub_days(31.0);
            self.mut_next_moon();
        }

        fn mut_next_moon(self: &mut Self) {
            let (mut dp_dt, mut last_phase) = (PhaseDiffState::Initial, 1.0);

            loop {
                let helio_earth = vsop87a::earth(self.as_jde_tai_days());
                let helio_earth_moon = vsop87a::earth_moon(self.as_jde_tai_days());
                let earth_lon = helio_earth.longitude();
                let moon_lon = helio_earth_moon.longitude();
                let diff = (earth_lon - moon_lon).abs();
                let phase = diff * 0.5 * std::f64::consts::FRAC_1_PI;

                //println!("Your phase is {} {:?} at {}", phase, dp_dt, hifitime::Epoch::from_jde_tai(self.clone()).as_gregorian_utc_str());

                let phasediff_signum = match (phase - &last_phase).signum() > 0.0 {
                    true => 1,
                    false => -1,
                };
                let (x, y) = match (&dp_dt, &phasediff_signum) {
                    (PhaseDiffState::Initial, _) => (PhaseDiffState::First, phase),
                    (PhaseDiffState::First, -1) => (PhaseDiffState::Negative, phase),
                    (PhaseDiffState::First, 1) => (PhaseDiffState::Positive, phase),
                    (PhaseDiffState::Positive, -1) => (PhaseDiffState::Negative, phase),
                    (PhaseDiffState::Positive, 1) => (PhaseDiffState::Positive, phase),
                    (PhaseDiffState::Negative, -1) => (PhaseDiffState::Negative, phase),
                    (PhaseDiffState::Negative, 1) => {
                        if helio_earth.magnitude() > helio_earth_moon.magnitude() {
                            // new moon
                            println!("Your new moon approx {}", self.as_gregorian_utc_str());
                            return;
                        } else {
                            // full moon
                            println!("Your full moon approx {}", self.as_gregorian_utc_str());
                            self.mut_add_days(13.0);
                            (PhaseDiffState::Positive, phase)
                        }
                    }
                    (_, _) => panic!(
                        "invalid combination of dp_dt {:?} and signum(phasediff) {}",
                        dp_dt, phasediff_signum
                    ),
                };
                dp_dt = x;
                last_phase = y;
                self.mut_add_secs(hifitime::SECONDS_PER_MINUTE);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        extern crate hifitime;
        use super::Epoch;
        use super::MoonFinder;

        const ERROR_TOLERANCE_JDE_20MIN: f64 = 20.0 * 60.0 / hifitime::SECONDS_PER_DAY;
        const ERROR_TOLERANCE_JDE_10MIN: f64 = 10.0 * 60.0 / hifitime::SECONDS_PER_DAY;

        // Currently has an error of about 13min and have seen as high as 20
        // Guessing as to why, perhaps due to omitting lunar libration
        #[test]
        fn test_aug_2020_new_moon_within_20min_tolerance() {
            let aug_2020_new_moon =
                Epoch::from_gregorian_utc(2020, 8, 19, 2, 28, 0, 0).as_jde_tai_days();
            let mut jde = Epoch::from_gregorian_utc(2020, 8, 18, 0, 0, 0, 0);
            jde.mut_next_moon();
            assert_eq!(
                (jde.as_jde_tai_days() - aug_2020_new_moon).abs() < ERROR_TOLERANCE_JDE_20MIN,
                true
            );
        }

        // Make sure the error is still between 10 and 20 min for this case
        #[test]
        fn test_aug_2020_new_moon_exceeds_10min_tolerance() {
            let aug_2020_new_moon =
                Epoch::from_gregorian_utc(2020, 8, 19, 2, 28, 0, 0).as_jde_tai_days();
            let mut jde = Epoch::from_gregorian_utc(2020, 8, 18, 0, 0, 0, 0);
            jde.mut_next_moon();
            assert_eq!(
                (jde.as_jde_tai_days() - aug_2020_new_moon).abs() < ERROR_TOLERANCE_JDE_10MIN,
                false
            );
        }
        #[test]
        fn test_aug_2020_last_moon_within_20min_tolerance() {
            let aug_2020_new_moon =
                Epoch::from_gregorian_utc(2020, 8, 19, 2, 28, 0, 0).as_jde_tai_days();
            let mut jde = Epoch::from_gregorian_utc(2020, 8, 28, 0, 0, 0, 0);
            jde.mut_last_moon();
            assert_eq!(
                (jde.as_jde_tai_days() - aug_2020_new_moon).abs() < ERROR_TOLERANCE_JDE_20MIN,
                true
            );
        }

        // Make sure the error is still between 10 and 20 min for this case
        #[test]
        fn test_aug_2020_last_moon_exceeds_10min_tolerance() {
            let aug_2020_new_moon =
                Epoch::from_gregorian_utc(2020, 8, 19, 2, 28, 0, 0).as_jde_tai_days();
            let mut jde = Epoch::from_gregorian_utc(2020, 8, 28, 0, 0, 0, 0);
            jde.mut_last_moon();
            assert_eq!(
                (jde.as_jde_tai_days() - aug_2020_new_moon).abs() < ERROR_TOLERANCE_JDE_10MIN,
                false
            );
        }
    }
}

pub mod tetrabiblos {
    extern crate hifitime;
    use super::moon::MoonFinder;

    #[derive(Debug)]
    pub enum Month {
        Capricornus,
        Aquarius,
        Pisces,
        Aries,
        Taurus,
        Gemini,
        Cancer,
        Leo,
        Virgo,
        Libra,
        Scorpius,
        Sagittarius,
    }

    pub trait ConvertibleMonth {
        fn convert_from_new_moon(newmoon: Epoch) -> Month {
            // First new moon in January has been Cap for at least all of Common Era
            // Before Cap, Aquarius, as in -3113 (new moon in Aquarius at JD ~= 584035.28)
            // Before Aqr, Pisces, as in -8238 (new moon in Pisces at JD ~= -1287850.72)
            //
            // Both of those are more or less in the center of their respective constellation
            // so, the change would have happened around the midpoint ~= -5700
        }
    }

    #[derive(Debug)]
    pub enum PrecessionalEraEpoch {
        // Yeah I'm the only one calling this -1 but, I'm not sure how else to differentiate
        // the first and second 13.0.0.0.0 because I do know 1.0.0.0.0.0 is something else
        // and it's like bowling, the first one only went up to 14.0.0.0.0 - 1
        // but the rest go to 20.0.0.0.0 - 1 I'm sure there is a way but it's too much
        // detail for Google search results to handle LMK.
        First,  // Mayan Long Count -1. 0.0.0.0.0   4  June -8238 Julian    JD = -1287717
        Second, // Mayan Long Count -1.13.0.0.0.0   6  Sep  -3113 Julian    JD = 584283
        Third,  // Mayan Long Count  0.13.0.0.0.0   21 Dec   2012 Gregorian JD = 2456283
    }

    const FIRST_EPOCH_JD: i64 = -1287717;
    const SECOND_EPOCH_JD: i64 = 584283;
    const THIRD_EPOCH_JD: i64 = 2456283;
    fn first_epoch_year_as_of_jan_2013() -> i32 {
        (((THIRD_EPOCH_JD - FIRST_EPOCH_JD) as f64) / hifitime::DAYS_PER_YEAR).floor()
            as i32
    }
    fn first_epoch_year_as_of_jan_2020() -> i32 {
        first_epoch_year_as_of_jan_2013() + 7 // 10257
    }
    fn second_epoch_year_as_of_jan_2013() -> i32 {
        (((THIRD_EPOCH_JD - SECOND_EPOCH_JD) as f64) / hifitime::DAYS_PER_YEAR).floor()
            as i32
    }
    fn second_epoch_year_as_of_jan_2020() -> i32 {
        second_epoch_year_as_of_jan_2013() + 7 // 5132
    }

    pub struct Date {
        pub epoch: PrecessionalEraEpoch,
        pub precessional_era: i8,
        pub year: i32,
        pub month: Month,
        pub day_of_month: i8,
    }

    impl Date {
        pub fn zero() -> Date {
            Date {
                epoch: PrecessionalEraEpoch::Second,
                precessional_era: 0,
                year: 0,
                month: Month::Capricornus,
                day_of_month: 0,
            }
        }
    }
    pub trait ConvertibleToTetrabiblos {
        fn to_tetrabiblos_date(&self) -> Date;
    }

    impl ConvertibleToTetrabiblos for hifitime::Epoch {
        fn to_tetrabiblos_date(&self) -> Date {
            let mut epoch = self.clone();
            epoch.mut_last_moon();
            let day_of_month = (epoch.as_utc_days() - self.as_utc_days()).ceil() as i8;
            Date {
                epoch: PrecessionalEraEpoch::Second,
                precessional_era: 0,
                year: 0,
                month: Month::Capricornus,
                day_of_month,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_years() {
            assert_eq!(first_epoch_year_as_of_jan_2013(), 10250);
            assert_eq!(first_epoch_year_as_of_jan_2020(), 10257);
            assert_eq!(second_epoch_year_as_of_jan_2013(), 5125);
            assert_eq!(second_epoch_year_as_of_jan_2020(), 5132);
        }
    }
}
