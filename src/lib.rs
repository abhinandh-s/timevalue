#![deny(rust_2018_idioms)]

use core::f64;

#[derive(PartialEq, Eq, Debug)]
pub enum TimeValueError {
    EmptyCashFlow,
    NegetiveDiscount,
}

pub trait TimeValue {
    fn present_value_regular(self) -> Result<f64, TimeValueError>;
    fn future_value_regular(self) -> Result<f64, TimeValueError>;
    fn present_value_due(self) -> Result<f64, TimeValueError>;
    fn future_value_due(self) -> Result<f64, TimeValueError>;
}

pub enum AnnuityKind {
    Regular,
    Due,
}

impl Default for AnnuityKind {
    fn default() -> Self {
        Self::Regular
    }
}

pub struct Annuity<T, I>
where
    I: IntoIterator<Item = T>,
    T: Into<f64> + Copy,
{
    cashflows: I,
    rate: f64,
    period: u32,
}

impl<T, I> Annuity<T, I>
where
    I: IntoIterator<Item = T>,
    T: Into<f64> + Copy,
{
    pub fn new(cashflows: I, rate: f64, period: u32) -> Self {
        Self {
            cashflows,
            rate,
            period,
        }
    }
}
fn round(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

impl<T, I> TimeValue for Annuity<T, I>
where
    I: IntoIterator<Item = T>,
    T: Into<f64> + Copy,
{
    fn present_value_regular(self) -> Result<f64, TimeValueError> {
        if self.rate < 0.0 {
            return Err(TimeValueError::NegetiveDiscount);
        }
        let cash_flows: Vec<_> = self.cashflows.into_iter().map(|i| i.into()).collect();
        if cash_flows.is_empty() {
            return Err(TimeValueError::EmptyCashFlow);
        }

        let present_value = cash_flows
            .iter()
            .enumerate()
            .map(|(i, &cash_flow)| cash_flow / (1.0 + self.rate).powi(i as i32 + 1))
            .sum::<f64>();

        Ok(round(present_value))
    }

    fn future_value_regular(self) -> Result<f64, TimeValueError> {
        if self.rate < 0.0 {
            return Err(TimeValueError::NegetiveDiscount);
        }
        let cash_flows: Vec<_> = self.cashflows.into_iter().map(|i| i.into()).collect();
        if cash_flows.is_empty() {
            return Err(TimeValueError::EmptyCashFlow);
        }

        let mut f: Vec<_> = cash_flows
            .iter()
            .enumerate()
            .map(|(i, &cash_flow)| cash_flow * (1.0 + self.rate).powi(i as i32 + 1))
            .collect();
        f.pop();
        if let Some(value) = cash_flows.last() {
            f.push(*value);
        }
        let future_value = f.iter().sum::<f64>();

        Ok(round(future_value))
    }

    fn present_value_due(self) -> Result<f64, TimeValueError> {
        let pv_r = Annuity::new(self.cashflows, self.rate, self.period).present_value_regular()?;
        Ok(round(pv_r * (1.0 + self.rate)))
    }

    fn future_value_due(self) -> Result<f64, TimeValueError> {
        if self.rate < 0.0 {
            return Err(TimeValueError::NegetiveDiscount);
        }
        let cash_flows: Vec<_> = self.cashflows.into_iter().map(|i| i.into()).collect();
        if cash_flows.is_empty() {
            return Err(TimeValueError::EmptyCashFlow);
        }

        let future_value = cash_flows
            .iter()
            .enumerate()
            .map(|(i, &cash_flow)| cash_flow * (1.0 + self.rate).powi(i as i32 + 1))
            .sum::<f64>();

        Ok(round(future_value))
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn annuity_pv() {
        let f = Annuity::new(
            [
                5_000, 5_000, 5_000, 5_000, 5_000, 5_000, 5_000, 5_000, 5_000, 5_000,
            ],
            0.12,
            10,
        );
        let pv = f.present_value_regular().unwrap();
        assert_eq!(pv, 28_251.12);
    }
    #[test]
    fn annuity_pv_due() {
        let f = Annuity::new([5_000].repeat(10), 0.12, 10);
        let pv = f.present_value_due().unwrap();
        assert_eq!(pv, 31641.25);
    }
    #[test]
    fn annuity_fv_reg() {
        let f = Annuity::new([50_000].repeat(7), 0.09, 7);
        let pv = f.future_value_regular().unwrap();
        assert_eq!(pv, 460021.73);
    }

    #[test]
    fn annuity_fv_due() {
        let f = Annuity::new(
            [
                200_000, 200_000, 200_000, 200_000, 200_000, 200_000, 200_000,
            ],
            0.12,
            7,
        );
        let pv = f.future_value_due().unwrap();
        assert_eq!(pv, 2_259_938.63);
    }
}
