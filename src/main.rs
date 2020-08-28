extern crate moonrs_lib;
use moonrs_lib::nextmoon;
extern crate hifitime;
use hifitime::Epoch;

pub fn main() {
    let mut jde = Epoch::from_gregorian_utc(2020, 8, 18, 0, 0, 0, 0);
    loop {
        nextmoon(&mut jde);
        jde.mut_add_secs(13.0 * hifitime::SECONDS_PER_DAY);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
