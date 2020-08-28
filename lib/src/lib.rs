extern crate hifitime;
extern crate vsop87;

use hifitime::Epoch;
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

pub fn nextmoon(jde: &mut Epoch) -> Epoch {
    let (mut dp_dt, mut last_phase, mut last_jde) = (PhaseDiffState::Initial, 1.0, 0.0);

    loop {
        let helio_earth = vsop87a::earth(jde.as_jde_tai_days());
        let helio_earth_moon = vsop87a::earth_moon(jde.as_jde_tai_days());
        let earth_lon = helio_earth.longitude();
        let moon_lon = helio_earth_moon.longitude();
        let diff = (earth_lon - moon_lon).abs();
        let phase = diff * 0.5 * std::f64::consts::FRAC_1_PI;

        //println!("Your phase is {} {:?} at {}", phase, dp_dt, hifitime::Epoch::from_jde_tai(jde.clone()).as_gregorian_utc_str());

        let phasediff_signum = match (phase - &last_phase).signum() > 0.0 {
            true => 1,
            false => -1,
        };
        let (x, y, z) = match (&dp_dt, &phasediff_signum) {
            (PhaseDiffState::Initial, _) => (PhaseDiffState::First, phase, jde.as_jde_tai_days()),
            (PhaseDiffState::First, -1) => (PhaseDiffState::Negative, phase, jde.as_jde_tai_days()),
            (PhaseDiffState::First, 1) => (PhaseDiffState::Positive, phase, jde.as_jde_tai_days()),
            (PhaseDiffState::Positive, -1) => {
                (PhaseDiffState::Negative, phase, jde.as_jde_tai_days())
            }
            (PhaseDiffState::Positive, 1) => {
                (PhaseDiffState::Positive, phase, jde.as_jde_tai_days())
            }
            (PhaseDiffState::Negative, -1) => {
                (PhaseDiffState::Negative, phase, jde.as_jde_tai_days())
            }
            (PhaseDiffState::Negative, 1) => {
                if helio_earth.magnitude() > helio_earth_moon.magnitude() {
                    // new moon
                    let e = Epoch::from_jde_tai(last_jde);
                    println!("Your new moon approx {}", e.as_gregorian_utc_str());
                    return e;
                } else {
                    // full moon
                    let e = Epoch::from_jde_tai(last_jde);
                    println!("Your full moon approx {}", e.as_gregorian_utc_str());
                    jde.mut_add_secs(13.0 * hifitime::SECONDS_PER_DAY);
                    (PhaseDiffState::Positive, phase, jde.as_jde_tai_days())
                }
            }
            (_, _) => panic!(
                "invalid combination of dp_dt {:?} and signum(phasediff) {}",
                dp_dt, phasediff_signum
            ),
        };
        dp_dt = x;
        last_phase = y;
        last_jde = z;
        jde.mut_add_secs(hifitime::SECONDS_PER_MINUTE);
    }
}

#[cfg(test)]
mod tests {
    use super::nextmoon;
    use super::Epoch;

    const ERROR_TOLERANCE_JDE_20MIN: f64 = 20.0 * 60.0 / hifitime::SECONDS_PER_DAY;
    const ERROR_TOLERANCE_JDE_10MIN: f64 = 10.0 * 60.0 / hifitime::SECONDS_PER_DAY;

    // Currently has an error of about 13min and have seen as high as 20
    // Guessing as to why, perhaps due to omitting lunar libration
    #[test]
    fn test_aug_2020_new_moon_within_20min_tolerance() {
        let aug_2020_new_moon =
            Epoch::from_gregorian_utc(2020, 8, 19, 2, 28, 0, 0).as_jde_tai_days();
        let mut jde = Epoch::from_gregorian_utc(2020, 8, 18, 0, 0, 0, 0);
        nextmoon(&mut jde);
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
        nextmoon(&mut jde);
        assert_eq!(
            (jde.as_jde_tai_days() - aug_2020_new_moon).abs() < ERROR_TOLERANCE_JDE_10MIN,
            false
        );
    }
}
