pub struct OrElse<It1, It2> {
    primary: It1,
    fallback: It2,
    primary_consumed: bool,
}

pub fn or<It1, It2>(primary: It1, fallback: It2) -> OrElse<It1, It2> {
    OrElse {
        primary,
        fallback,
        primary_consumed: false,
    }
}

impl<It1, It2> Iterator for OrElse<It1, It2>
where
    It1: Iterator,
    It2: Iterator<Item = It1::Item>,
{
    type Item = It1::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.primary.next() {
            self.primary_consumed = true;
            Some(item)
        } else if !self.primary_consumed {
            self.fallback.next()
        } else {
            None
        }
    }
}

pub trait ItWithFallback: Iterator {
    fn or<It>(self, fallback: It) -> OrElse<Self, It>
    where
        It: IntoIterator<Item = Self::Item>,
        Self: Sized,
    {
        or(self, fallback)
    }
}

impl<T> ItWithFallback for T where T: Iterator {}
