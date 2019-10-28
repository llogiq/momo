use momo::momo;

#[momo]
fn check_generic<I: Into<usize>, S: AsRef<str>, M: AsMut<[usize]>>(i: I, s: S, mut m: M) -> usize {
    i.into() + s.as_ref().len() + m.as_mut()[0]
}

#[momo]
fn check_impl_trait(i: impl Into<usize>, s: impl AsRef<str>, mut m: impl AsMut<[usize]>) -> usize {
    i.into() + s.as_ref().len() + m.as_mut()[0]
}

fn main() {
    let i = 1u8;
    let s = "Hello, Rust!";
    let mut m = [3usize, 42];
    assert_eq!(check_generic(i, s, &mut m), 16);
    assert_eq!(check_impl_trait(i, s, &mut m), 16);
}

