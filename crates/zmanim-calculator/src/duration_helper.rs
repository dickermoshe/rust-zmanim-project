use chrono::TimeDelta;
use core::time::Duration as StdDuration;
use time::Duration as TimeDuration;

/// A helper function to multiply a duration by a factor.
/// This uses a clever workaround to handle negative durations which std duration does not support.
pub(crate) fn multiply_duration(core_timedelta: TimeDelta, factor: f64) -> Option<TimeDelta> {
    let is_timedelta_negative = core_timedelta < TimeDelta::zero();
    let factor_is_negative = factor < 0.0;
    let std_duration = core_timedelta.abs().to_std().ok()?;
    let time_duration: TimeDuration = std_duration.try_into().ok()?;
    let std_duration: StdDuration = (time_duration * factor.abs()).try_into().ok()?;
    let core_timedelta = TimeDelta::from_std(std_duration).ok()?;

    if (is_timedelta_negative && !factor_is_negative)
        || (!is_timedelta_negative && factor_is_negative)
    {
        core_timedelta.checked_mul(-1)
    } else {
        Some(core_timedelta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiply_duration_positive_duration_positive_factor() {
        let duration = TimeDelta::seconds(100);
        let result = multiply_duration(duration, 2.0);
        assert_eq!(result, Some(TimeDelta::seconds(200)));
    }

    #[test]
    fn test_multiply_duration_positive_duration_negative_factor() {
        let duration = TimeDelta::seconds(100);
        let result = multiply_duration(duration, -2.0);
        assert_eq!(result, Some(TimeDelta::seconds(-200)));
    }

    #[test]
    fn test_multiply_duration_negative_duration_positive_factor() {
        let duration = TimeDelta::seconds(-100);
        let result = multiply_duration(duration, 2.0);
        assert_eq!(result, Some(TimeDelta::seconds(-200)));
    }

    #[test]
    fn test_multiply_duration_negative_duration_negative_factor() {
        let duration = TimeDelta::seconds(-100);
        let result = multiply_duration(duration, -2.0);
        assert_eq!(result, Some(TimeDelta::seconds(200)));
    }

    #[test]
    fn test_multiply_duration_zero_duration() {
        let duration = TimeDelta::zero();
        let result = multiply_duration(duration, 5.0);
        assert_eq!(result, Some(TimeDelta::zero()));

        let result_negative = multiply_duration(duration, -5.0);
        assert_eq!(result_negative, Some(TimeDelta::zero()));
    }

    #[test]
    fn test_multiply_duration_zero_factor() {
        let duration = TimeDelta::seconds(100);
        let result = multiply_duration(duration, 0.0);
        assert_eq!(result, Some(TimeDelta::zero()));

        let negative_duration = TimeDelta::seconds(-100);
        let result_negative = multiply_duration(negative_duration, 0.0);
        assert_eq!(result_negative, Some(TimeDelta::zero()));
    }

    #[test]
    fn test_multiply_duration_identity_factor() {
        let duration = TimeDelta::seconds(100);
        let result = multiply_duration(duration, 1.0);
        assert_eq!(result, Some(duration));

        let negative_duration = TimeDelta::seconds(-100);
        let result_negative = multiply_duration(negative_duration, 1.0);
        assert_eq!(result_negative, Some(negative_duration));
    }

    #[test]
    fn test_multiply_duration_negation_factor() {
        let duration = TimeDelta::seconds(100);
        let result = multiply_duration(duration, -1.0);
        assert_eq!(result, Some(TimeDelta::seconds(-100)));

        let negative_duration = TimeDelta::seconds(-100);
        let result_negative = multiply_duration(negative_duration, -1.0);
        assert_eq!(result_negative, Some(TimeDelta::seconds(100)));
    }

    #[test]
    fn test_multiply_duration_fractional_factors() {
        let duration = TimeDelta::seconds(100);
        let result = multiply_duration(duration, 0.5);
        assert_eq!(result, Some(TimeDelta::seconds(50)));

        let result = multiply_duration(duration, 1.5);
        assert_eq!(result, Some(TimeDelta::seconds(150)));

        let result = multiply_duration(duration, -0.5);
        assert_eq!(result, Some(TimeDelta::seconds(-50)));
    }

    #[test]
    fn test_multiply_duration_millisecond_precision() {
        let duration = TimeDelta::milliseconds(123);
        let result = multiply_duration(duration, 2.0);
        assert_eq!(result, Some(TimeDelta::milliseconds(246)));

        let result = multiply_duration(duration, -3.0);
        assert_eq!(result, Some(TimeDelta::milliseconds(-369)));
    }

    #[test]
    fn test_multiply_duration_hours() {
        let duration = TimeDelta::hours(1);
        let result = multiply_duration(duration, 3.0);
        assert_eq!(result, Some(TimeDelta::hours(3)));

        let result = multiply_duration(duration, 0.5);
        assert_eq!(result, Some(TimeDelta::minutes(30)));
    }

    #[test]
    fn test_multiply_duration_minutes() {
        let duration = TimeDelta::minutes(72);
        let result = multiply_duration(duration, 2.0);
        assert_eq!(result, Some(TimeDelta::minutes(144)));

        let result = multiply_duration(duration, -1.0);
        assert_eq!(result, Some(TimeDelta::minutes(-72)));
    }

    #[test]
    fn test_multiply_duration_days() {
        let duration = TimeDelta::days(1);
        let result = multiply_duration(duration, 7.0);
        assert_eq!(result, Some(TimeDelta::days(7)));

        let result = multiply_duration(duration, -0.5);
        assert_eq!(result, Some(TimeDelta::hours(-12)));
    }
}
