pub struct OnceGet<T> {
    t: T,
}

impl<T> OnceGet<T> {
    pub fn new(f: fn() -> T) -> OnceGet<T> {
        OnceGet { t: f() }
    }

    pub fn get(&self) -> &T {
        &self.t
    }
}
