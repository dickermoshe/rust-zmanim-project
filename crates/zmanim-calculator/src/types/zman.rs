use chrono::{DateTime, TimeZone, Utc};

use crate::calculator::ZmanimCalculator;

pub trait ZmanLike {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>>;
    #[cfg(test)]
    fn uses_elevation(&self) -> bool;
    #[cfg(test)]
    fn java_function_name(&self) -> &str;
}
