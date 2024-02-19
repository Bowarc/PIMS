

#[test]
fn scan(){
    assert_eq!(1,1);
}
#[test]
fn read(){

    use std::mem;

    struct T { a: i32, b: i32 }

    let t = T { a: 17456745, b: i32::MAX };
    println!("{} bytes of T:", mem::size_of::<T>());
    let view = &t as *const _ as *const u8;
    for i in 0.. mem::size_of::<T>() as i32 {
        print!("{:02x} ", unsafe {*view.offset(i.try_into().unwrap())});
    }
    
    println!("\n10 bytes of main");
    let view = read as *const u8;
    for i in 0..10 {
        print!("{:02x} ", unsafe {*view.offset(i)});
    }


}