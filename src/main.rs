extern crate moonrs_lib;
extern crate vsop87;
extern crate hifitime;

use moonrs_lib::nextmoons;
use hifitime::Epoch;

pub fn main() {
    let mut jde = Epoch::from_gregorian_utc(2020, 8, 20, 0, 0, 0, 0).as_jde_utc_days();
    loop {
        jde = nextmoons(jde);
        jde += 1.0/24.0;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
