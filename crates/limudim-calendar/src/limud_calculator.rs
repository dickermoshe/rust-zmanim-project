use crate::{cycle::Cycle, date::HebrewDate, interval::Interval};

pub type PerpetualCycleFinder = fn(HebrewDate) -> (HebrewDate, HebrewDate);

pub enum CycleFinder {
    Initial(HebrewDate),
    Perpetual(PerpetualCycleFinder),
}

pub trait LimudCalculator<T> {
    fn limud(&self, limud_date: HebrewDate) -> Option<T> {
        let cycle = self.find_cycle(limud_date)?;
        if cycle.end_date < limud_date {
            return None;
        }
        let mut interval = Interval::first_for_cycle(cycle, Self::interval_end_calculation)?;
        while !interval.contains(limud_date) {
            interval = if self.is_skip_interval(&interval) {
                interval.skip(Self::interval_end_calculation)?
            } else {
                interval.next(Self::interval_end_calculation)?
            };
        }
        if self.is_skip_interval(&interval) {
            return None;
        }
        self.unit_for_interval(&interval, &limud_date)
    }
    fn cycle_finder(&self) -> CycleFinder;
    fn find_cycle(&self, date: HebrewDate) -> Option<Cycle> {
        match self.cycle_finder() {
            CycleFinder::Initial(initial_cycle_date) => {
                Cycle::from_cycle_initiation(initial_cycle_date, Self::cycle_end_calculation, date)
            }
            CycleFinder::Perpetual(finder) => Some(Cycle::from_perpetual(finder, date)),
        }
    }
    fn cycle_end_calculation(hebrew_date: HebrewDate, _iteration: Option<i32>) -> Option<HebrewDate> {
        Some(hebrew_date)
    }
    fn interval_end_calculation(_cycle: Cycle, hebrew_date: HebrewDate) -> Option<HebrewDate> {
        Some(hebrew_date)
    }
    fn is_skip_interval(&self, _interval: &Interval) -> bool {
        false
    }
    fn unit_for_interval(&self, interval: &Interval, limud_date: &HebrewDate) -> Option<T>;
}
