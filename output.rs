struct Foo {
    value: i32,
}

impl Foo {
    fn new(v: i32) -> Self {
        Self { value: v }
    }
}

fn main() {
    let f = Foo::new(1);
    println!("{}", f.value);
}
