pub fn then<T, K, F: FnMut(T)>(result: Result<T, K>, mut closure: F, message: &str) {
    match result {
        Ok(val) => {
            closure(val);
        }
        Err(_) => println!("{}", message),
    };
}
