use std::cell::Cell;

pub trait OptionCellExt<T> {
    fn with<R, F: FnOnce(&mut T) -> R>(&self, func: F) -> Option<R>;
}

impl<T> OptionCellExt<T> for Cell<Option<T>> {
    fn with<R, F: FnOnce(&mut T) -> R>(&self, func: F) -> Option<R> {
        self
            .take()
            .map(|mut inner| {
                let result = func(&mut inner);
                self.set(Some(inner));
                result
            })
    }
}