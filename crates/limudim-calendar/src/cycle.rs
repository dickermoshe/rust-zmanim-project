use crate::{date::*, limud_calculator::PerpetualCycleFinder};
pub type CycleEndCalculation = fn(HebrewDate, Option<i32>) -> Option<HebrewDate>;

#[derive(Clone, Copy)]
pub struct Cycle {
    pub start_date: HebrewDate,
    pub end_date: HebrewDate,
    pub iteration: Option<i32>,
}
impl Cycle {
    pub fn from_perpetual(finder: PerpetualCycleFinder, date: HebrewDate) -> Self {
        let (start_date, end_date) = finder(date);
        Self {
            start_date,
            end_date,
            iteration: None,
        }
    }
    pub fn from_cycle_initiation(
        initial_cycle_date: HebrewDate,
        cycle_end_calculation: CycleEndCalculation,
        date: HebrewDate,
    ) -> Option<Self> {
        if initial_cycle_date > date {
            return None;
        }
        let iteration = 1;
        let end_date = cycle_end_calculation(initial_cycle_date, Some(iteration))?;
        let mut cycle = Self {
            start_date: initial_cycle_date,
            end_date,
            iteration: Some(iteration),
        };
        while date > cycle.end_date {
            cycle = cycle.next(cycle_end_calculation)?;
        }
        Some(cycle)
    }

    pub fn next(&self, cycle_end_calculation: CycleEndCalculation) -> Option<Self> {
        if let Some(iteration) = self.iteration {
            let new_iteration = iteration + 1;
            let new_start_date = self.end_date.add_days(1)?;
            let new_end_date = cycle_end_calculation(new_start_date, Some(new_iteration))?;
            Some(Self {
                start_date: new_start_date,
                end_date: new_end_date,
                iteration: Some(new_iteration),
            })
        } else {
            None
        }
    }
}
