use std::cmp::Ordering;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tick {
    pub value: f32,
}

impl Tick {
    pub const fn from_duration(duration: Duration, ups: f32) -> Self {
        let value = (duration.as_millis() as f32 / 1000.0) * ups;
        Self { value }
    }

    pub fn as_duration_with_ups(&self, ups: f32) -> Duration {
        Duration::from_millis(((self.value / ups) * 1000.0) as u64)
    }
}

impl PartialOrd for Tick {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

#[cfg(test)]
mod tests {
    use crate::tick::Tick;
    use std::time::Duration;

    #[test]
    fn ticks_from_seconds_1() {
        let ups = 20.0;
        let tick = Tick::from_duration(Duration::from_secs(4), ups);
        assert_eq!(tick.value, 80.0)
    }

    #[test]
    fn ticks_from_seconds_2() {
        let ups = 1.0;
        let tick = Tick::from_duration(Duration::from_secs(4), ups);
        assert_eq!(tick.value, 4.0)
    }

    #[test]
    fn ticks_from_millis_1() {
        let ups = 20.0;
        let tick = Tick::from_duration(Duration::from_millis(200), ups);
        assert_eq!(tick.value, 4.0)
    }

    #[test]
    fn ticks_from_millis_2() {
        let ups = 20.0;
        let tick = Tick::from_duration(Duration::from_millis(1000), ups);
        assert_eq!(tick.value, 20.0)
    }

    #[test]
    fn ticks_from_millis_3() {
        let ups = 20.0;
        let tick = Tick::from_duration(Duration::from_millis(50), ups);
        assert_eq!(tick.value, 1.0)
    }

    #[test]
    fn ticks_from_millis_4() {
        let ups = 1.0;
        let tick = Tick::from_duration(Duration::from_millis(200), ups);
        assert_eq!(tick.value, 0.2)
    }

    #[test]
    fn ticks_from_millis_5() {
        let ups = 1.0;
        let tick = Tick::from_duration(Duration::from_millis(1000), ups);
        assert_eq!(tick.value, 1.0)
    }

    #[test]
    fn ticks_from_millis_6() {
        let ups = 1.0;
        let tick = Tick::from_duration(Duration::from_millis(50), ups);
        assert_eq!(tick.value, 0.05)
    }

    #[test]
    fn duration_from_ticks_1() {
        let ups = 20.0;
        let tick = Tick { value: 1.0 };
        let millis = tick.as_duration_with_ups(ups).as_millis();
        assert_eq!(millis, 50)
    }

    #[test]
    fn duration_from_ticks_2() {
        let ups = 20.0;
        let tick = Tick { value: 0.2 };
        let millis = tick.as_duration_with_ups(ups).as_millis();
        assert_eq!(millis, 10)
    }

    #[test]
    fn duration_from_ticks_3() {
        let ups = 20.0;
        let tick = Tick { value: 0.1 };
        let millis = tick.as_duration_with_ups(ups).as_millis();
        assert_eq!(millis, 5)
    }
}
