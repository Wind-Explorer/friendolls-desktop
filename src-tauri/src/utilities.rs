#[macro_export]
macro_rules! lock_r {
    ($rwlock:expr) => {{
        match $rwlock.read() {
            Ok(guard) => guard,
            Err(_) => panic!(
                "Failed to acquire read lock on {} at {}:{}",
                stringify!($rwlock),
                file!(),
                line!()
            ),
        }
    }};
}

#[macro_export]
macro_rules! lock_w {
    ($rwlock:expr) => {{
        match $rwlock.write() {
            Ok(guard) => guard,
            Err(_) => panic!(
                "Failed to acquire write lock on {} at {}:{}",
                stringify!($rwlock),
                file!(),
                line!()
            ),
        }
    }};
}
