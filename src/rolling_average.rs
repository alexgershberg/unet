use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct RollingAverage {
    pub(crate) values: VecDeque<f32>,
    value: f32,
    n: usize,
}

impl RollingAverage {
    pub fn new(n: usize) -> Self {
        let values = VecDeque::from(vec![0.0; n]);
        Self {
            values,
            value: 0.0,
            n,
        }
    }

    pub fn add(&mut self, value: f32) {
        self.values.push_front(value);
        self.values.pop_back();

        assert_eq!(self.values.len(), self.n)
    }

    pub fn value(&self) -> f32 {
        let mut avg = 0.0;
        for i in &self.values {
            avg += i;
        }

        avg /= self.n as f32;

        avg
    }
}

#[cfg(test)]
mod tests {
    use crate::rolling_average::RollingAverage;

    #[test]
    fn rolling_average_1() {
        let mut rolling_average = RollingAverage::new(5);

        rolling_average.add(10.0);
        assert_eq!(rolling_average.value(), 2.0);
        rolling_average.add(0.0);
        assert_eq!(rolling_average.value(), 2.0);
        rolling_average.add(20.0);
        assert_eq!(rolling_average.value(), 6.0);
        rolling_average.add(2.0);
        assert_eq!(rolling_average.value(), 6.4);
        rolling_average.add(15.0);
        assert_eq!(rolling_average.value(), 9.4);

        rolling_average.add(3.0);
        assert_eq!(rolling_average.value(), 8.0);
    }
}
