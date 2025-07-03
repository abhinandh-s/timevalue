#![allow(dead_code, unused)]
#![deny(rust_2018_idioms)]

use core::f64;
use std::marker::PhantomData;

pub trait TimeValue {
    fn present_value(&self) -> Result<f64, ValueError>;
    fn future_value(&self) -> Result<f64, ValueError>;
}

fn round(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

pub struct SingleSum<T>
where
    T: Into<f64> + Copy,
{
    amount: T,
    rate: f64,
    period: u32,
}

impl<T> SingleSum<T>
where
    T: Into<f64> + Copy,
{
    fn new(amt: T, rate: f64, period: u32) -> Self {
        Self {
            amount: amt,
            rate,
            period,
        }
    }

    pub fn amount(&self) -> T {
        self.amount
    }

    pub fn rate(&self) -> f64 {
        self.rate
    }

    pub fn period(&self) -> u32 {
        self.period
    }

    pub fn set_amount(&mut self, amt: T) {
        self.amount = amt;
    }

    pub fn set_rate(&mut self, rate: f64) {
        self.rate = rate;
    }

    pub fn set_period(&mut self, period: u32) {
        self.period = period;
    }
}

impl<T> TimeValue for SingleSum<T>
where
    T: Into<f64> + Copy,
{
    fn present_value(&self) -> Result<f64, ValueError> {
        if self.rate < 0.0 {
            return Err(ValueError::NegativeDiscount);
        }

        let pv = self.amount.into() / (1.0 + self.rate).powi(self.period as i32);
        Ok(round(pv))
    }

    fn future_value(&self) -> Result<f64, ValueError> {
        if self.rate < 0.0 {
            return Err(ValueError::NegativeDiscount);
        }

        let fv = self.amount.into() * (1.0 + self.rate).powi(self.period as i32);
        Ok(round(fv))
    }
}

pub struct Annuity<T, M>
where
    T: Into<f64> + Copy,
{
    cashflow: T,
    rate: f64,
    period: u32,
    _marker: PhantomData<M>,
}

impl<T, M> Annuity<T, M>
where
    T: Into<f64> + Copy,
{
    pub fn new(cashflows: T, rate: f64, period: u32) -> Self {
        Self {
            cashflow: cashflows,
            rate,
            period,
            _marker: PhantomData,
        }
    }
}

pub struct Regular {}
pub struct Due {}

impl<T> TimeValue for Annuity<T, Due>
where
    T: Into<f64> + Copy,
{
    fn present_value(&self) -> Result<f64, ValueError> {
        if self.rate < 0.0 {
            return Err(ValueError::NegativeDiscount);
        }
        let pv_r: Annuity<T, Regular> = Annuity::new(self.cashflow, self.rate, self.period);
        let pv = pv_r.present_value().unwrap();
        let res = pv * (1.0 + self.rate);
        Ok(round(res))
    }

    fn future_value(&self) -> Result<f64, ValueError> {
        if self.rate < 0.0 {
            return Err(ValueError::NegativeDiscount);
        }
        let factor = |rate: f64, period: u32| -> f64 {
            let f = (1.0 + rate);
            let mut res = f;
            let mut result = Vec::new();
            let count = 10;
            for i in 0..period {
                result.push(res);
                res *= f;
            }
            debug_assert_eq!(period as usize, result.len());
            result.iter().sum()
        };

        let f = factor(self.rate, self.period);
        let pv = self.cashflow.into() * f;

        Ok(round(pv))
    }
}

impl<T> TimeValue for Annuity<T, Regular>
where
    T: Into<f64> + Copy,
{
    fn present_value(&self) -> Result<f64, ValueError> {
        if self.rate < 0.0 {
            return Err(ValueError::NegativeDiscount);
        }

        let factor = |rate: f64, period: u32| -> f64 {
            let f = 1.0 / (1.0 + rate);
            let mut res = f;
            let mut result = Vec::new();
            let count = 10;
            for i in 0..period {
                result.push(res);
                res *= f;
            }
            debug_assert_eq!(period as usize, result.len());
            result.iter().sum()
        };

        let f = factor(self.rate, self.period);
        let pv = self.cashflow.into() * f;

        Ok(round(pv))
    }

    fn future_value(&self) -> Result<f64, ValueError> {
        if self.rate < 0.0 {
            return Err(ValueError::NegativeDiscount);
        }

        let factor = |rate: f64, period: u32| -> f64 {
            let f = (1.0 + rate);
            let mut res = f;
            let mut result = Vec::new();
            let count = 10;
            for i in 0..period - 1 {
                result.push(res);
                res *= f;
            }
            debug_assert_eq!(period as usize, result.len() + 1);
            result.iter().sum()
        };

        let f = factor(self.rate, self.period) + 1.0;
        let pv = self.cashflow.into() * f;

        Ok(round(pv))
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum ValueError {
    NegativeDiscount,
    EmptyCashFlow,
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

#[cfg(test)]
mod tests {
    use std::fmt::Result;

    use super::*;

    #[test]
    fn single_sum_fv() {
        let single_sum = SingleSum::new(150_000, 0.12, 10);
        let fv = single_sum.future_value().unwrap();
        assert_eq!(fv, 465877.23);
    }

    #[test]
    fn single_sum_pv() {
        let f = SingleSum::new(1_000, 0.10, 3);
        let pv = f.present_value().unwrap();
        assert_eq!(pv, 751.31);
    }
    #[test]
    fn annuity_pv() {
        let f: Annuity<i32, Regular> = Annuity::new(5_000, 0.12, 10);
        let pv = f.present_value().unwrap();
        assert_eq!(pv, 28_251.12);
    }
    #[test]
    fn annuity_pv_due() {
        let f: Annuity<i32, Due> = Annuity::new(5_000, 0.12, 10);
        let pv = f.present_value().unwrap();
        assert_eq!(pv, 31641.25);
    }

    #[test]
    fn annuity_fv_reg() {
        let f: Annuity<i32, Regular> = Annuity::new(50_000, 0.09, 7);
        let pv = f.future_value().unwrap();
        assert_eq!(pv, 460021.73);
    }
    
    #[test]
    fn annuity_fv_due() {
        let f: Annuity<i32, Due> = Annuity::new(200_000, 0.12, 7);
        let pv = f.future_value().unwrap();
        assert_eq!(pv, 2_259_938.63);
    }
}
