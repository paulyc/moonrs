extern crate vsop87;
extern crate hifitime;

use vsop87::RectangularCoordinates;
use vsop87::vsop87a;
use hifitime::Epoch;

pub fn gregorian_to_jde(year: i32, month: u8, day: u8, hour: u8, min: u8, sec: u8, nsec: u32) -> f64 {
    Epoch::from_gregorian_utc(year, month, day, hour, min, sec, nsec).as_jde_utc_days()
}

pub trait ConvertibleToSpherical {
    fn longitude(&self) -> f64 ;
    fn magnitude(&self) -> f64;
}

impl ConvertibleToSpherical for RectangularCoordinates {
    fn longitude(&self) -> f64 {
        (self.y.clone()/self.x.clone()).atan()
    }
    fn magnitude(&self) -> f64 {
        (self.x.clone()*self.x.clone() + self.y.clone()*self.y.clone() + self.z.clone()*self.z.clone()).sqrt()
    }
}

#[derive(Debug)]
enum PhaseDerivative { Void, Unknown, Positive, Negative }

pub fn nextmoons(mut jde: f64) -> f64 {
    let mut dp_dt = PhaseDerivative::Unknown;
    let mut last_phase = 1.0;
    let mut last_jde = 1.0;
    loop {
        let helio_earth = vsop87a::earth(jde.clone());
        let helio_earth_moon = vsop87a::earth_moon(jde.clone());
        let earth_lon = helio_earth.longitude();
        let moon_lon  = helio_earth_moon.longitude();
        let diff = (earth_lon - moon_lon).abs();
        let phase = diff * 0.5 * std::f64::consts::FRAC_1_PI;

        //println!("Your phase is {} {:?} at {}", phase, dp_dt, hifitime::Epoch::from_jde_tai(jde.clone()).as_gregorian_utc_str());

        let phasediff_signum = match (phase-last_phase.clone()).signum() > 0.0 {
            true => 1,
            false => -1,
        };
        let (x, y, z) = match (&dp_dt, &phasediff_signum) {
            (PhaseDerivative::Unknown, _) => (PhaseDerivative::Void, phase, jde.clone()),
            (PhaseDerivative::Void, -1) => (PhaseDerivative::Negative, phase, jde.clone()),
            (PhaseDerivative::Void, 1) => (PhaseDerivative::Positive, phase, jde.clone()),
            (PhaseDerivative::Positive, -1) => (PhaseDerivative::Negative, phase, jde.clone()),
            (PhaseDerivative::Positive, 1) =>  (PhaseDerivative::Positive, phase, jde.clone()),
            (PhaseDerivative::Negative, -1) => (PhaseDerivative::Negative, phase, jde.clone()),
            (PhaseDerivative::Negative, 1) => {
                if helio_earth.magnitude() > helio_earth_moon.magnitude() {
                    // new moon
                    println!("Your new moon approx {}", hifitime::Epoch::from_jde_tai(last_jde).as_gregorian_utc_str());
                    return last_jde;
                } else {
                    // full moon
                    println!("Your full moon approx {}", hifitime::Epoch::from_jde_tai(last_jde).as_gregorian_utc_str());
                    (PhaseDerivative::Positive, phase, jde.clone())
                }
            },
            (_, _) => panic!("invalid combination of dp_dt {:?} and signum(phasediff) {}", dp_dt, phasediff_signum)
        };
        dp_dt = x;
        last_phase = y;
        last_jde = z;
        jde += 1.0/(24.0*60.0); // 1min
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
