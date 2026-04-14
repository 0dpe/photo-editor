// on web, expect() or unwrap() are both not recommended
// instead, wasm_bindgen's expect_throw() or unwrap_throw() are recommended, which throws a JavaScript error
// however, always writing two versions of the same code, one with expect() and one with expect_throw(), is tedious
// so, just always use expect_universal() defined here
pub trait ExpectUniversal<T> {
    fn expect_universal(self, message: &str) -> T;
}

#[cfg(target_arch = "wasm32")]
impl<T> ExpectUniversal<T> for Option<T> {
    fn expect_universal(self, message: &str) -> T {
        use wasm_bindgen::UnwrapThrowExt;
        self.expect_throw(message)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T> ExpectUniversal<T> for Option<T> {
    fn expect_universal(self, message: &str) -> T {
        self.expect(message)
    }
}

#[cfg(target_arch = "wasm32")]
impl<T, E: core::fmt::Debug> ExpectUniversal<T> for Result<T, E> {
    fn expect_universal(self, message: &str) -> T {
        use wasm_bindgen::UnwrapThrowExt;
        self.expect_throw(message)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T, E: core::fmt::Debug> ExpectUniversal<T> for Result<T, E> {
    fn expect_universal(self, message: &str) -> T {
        self.expect(message)
    }
}
