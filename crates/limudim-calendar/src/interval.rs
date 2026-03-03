use crate::{
    cycle::Cycle,
    date::{DateExt, HebrewDate},
};

pub type IntervalEndCalculation = fn(Cycle, HebrewDate) -> Option<HebrewDate>;

pub struct Interval {
    pub start_date: HebrewDate,
    pub end_date: HebrewDate,
    pub iteration: i32,
    pub cycle: Cycle,
}
impl Interval {
    pub fn first_for_cycle(cycle: Cycle, interval_end_calculation: IntervalEndCalculation) -> Option<Self> {
        let start_date = cycle.start_date;
        let iteration = 1;
        let end_date = interval_end_calculation(cycle, start_date)?;
        Some(Self {
            start_date,
            end_date,
            iteration,
            cycle,
        })
    }
    pub fn next(&self, interval_end_calculation: IntervalEndCalculation) -> Option<Self> {
        self._next_for_iteration(self.iteration + 1, interval_end_calculation)
    }
    pub fn skip(&self, interval_end_calculation: IntervalEndCalculation) -> Option<Self> {
        self._next_for_iteration(self.iteration, interval_end_calculation)
    }
    fn _next_for_iteration(
        &self,
        new_iteration: i32,
        interval_end_calculation: IntervalEndCalculation,
    ) -> Option<Self> {
        if self.end_date >= self.cycle.end_date {
            return None;
        }
        let new_start_date = self.end_date.add_days(1)?;
        let new_end_date = interval_end_calculation(self.cycle, new_start_date)?;
        Some(Self {
            start_date: new_start_date,
            end_date: new_end_date,
            iteration: new_iteration,
            cycle: self.cycle,
        })
    }
    pub fn contains(&self, date: HebrewDate) -> bool {
        self.start_date <= date && date <= self.end_date
    }
}
