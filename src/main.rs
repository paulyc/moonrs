extern crate tetrabiblos_lib;
use tetrabiblos_lib::moon::MoonFinder;

extern crate hifitime;
use hifitime::Epoch;

pub fn main() {
    let mut jde = Epoch::from_gregorian_utc(2020, 8, 18, 0, 0, 0, 0);
    loop {
        jde.mut_next_moon();
        jde.mut_add_days(13.0);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
