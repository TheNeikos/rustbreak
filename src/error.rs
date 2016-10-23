
quick_error! {
    /// The Error type exported by BreakError, usually you only need to check against NotFound,
    /// however it might be useful sometimes to get other errors.
    #[derive(Debug)]
    pub enum BreakError {
        /// An error returned when doing file operations, this might happen by opening, closing,
        /// locking or flushing
        Io(err: ::std::io::Error) {
            from()
        }
        /// This error happens if Bincode cannot deserialize a given file. If you get this error
        /// check your database is not corrupt. (This includes non-empty files **not** created by
        /// RustBreak!
        Deserialize(err: ::bincode::serde::DeserializeError) {
            from()
        }
        /// This error happens if bincode cannot serialize the given type at runtime
        Serialize(err: ::bincode::serde::SerializeError) {
            from()
        }
        /// Poisoned, you can recover from this by running `recover_poison` on the database
        Poison {}
        /// This simply means your key could not be found in the database
        NotFound {}
    }
}

impl<T> From<::std::sync::PoisonError<T>> for BreakError {
    fn from(_: ::std::sync::PoisonError<T>) -> BreakError {
        BreakError::Poison
    }
}
