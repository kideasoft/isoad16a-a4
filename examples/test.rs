use isoad16a_a4::Ad;

fn main() {
    let mut ad = Ad::new("/dev/ttyUSB0", 0x01, 9600).unwrap();

    for _ in 0..100 {
        let r = ad.get_all().unwrap();
        println!("AD values: {:?}", r);
    }
}
