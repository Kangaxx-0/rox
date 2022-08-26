use rox::lec::Lec;

#[test]
fn new() {
    let lec: Lec<u8> = Lec::new();

    assert_eq!(0, lec.len());
    assert_eq!(0, lec.capacity());
}

#[test]
fn push_and_pop() {
    let mut lec: Lec<u8> = Lec::new();

    assert_eq!(0, lec.len());
    assert_eq!(0, lec.capacity());
    lec.push(1);
    assert_eq!(1, lec.len());
    assert_eq!(1, lec.capacity());
    lec.push(2);
    assert_eq!(2, lec.len());
    assert_eq!(2, lec.capacity());
    lec.push(3);
    assert_eq!(3, lec.len());
    assert_eq!(4, lec.capacity());
    let value = lec.pop();
    assert_eq!(3, value.unwrap());
    assert_eq!(2, lec.len());
    assert_eq!(4, lec.capacity());
}
