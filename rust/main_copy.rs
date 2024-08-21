use std::error::Error;
use std::fmt;
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
struct FinancialParams {
    num_years: i32,
    au_hours: f64,
    initial_tsn: f64,
    rate_escalation: f64,
    aic: f64,
    hsi_tsn: f64,
    overhaul_tsn: f64,
    hsi_cost: f64,
    overhaul_cost: f64,
}

impl FinancialParams {
    fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.num_years <= 0 {
            return Err("NumYears must be positive".into());
        }
        if self.au_hours <= 0.0 {
            return Err("AuHours must be positive".into());
        }
        if self.initial_tsn < 0.0 {
            return Err("InitialTSN cannot be negative".into());
        }
        if self.rate_escalation < 0.0 {
            return Err("RateEscalation cannot be negative".into());
        }
        if !(0.0..=100.0).contains(&self.aic) {
            return Err("AIC must be between 0 and 100".into());
        }
        if self.hsi_tsn <= 0.0 {
            return Err("HSITSN must be positive".into());
        }
        if self.overhaul_tsn <= 0.0 {
            return Err("OverhaulTSN must be positive".into());
        }
        if self.hsi_cost < 0.0 {
            return Err("HSICost cannot be negative".into());
        }
        if self.overhaul_cost < 0.0 {
            return Err("OverhaulCost cannot be negative".into());
        }
        Ok(())
    }
}

#[derive(Debug)]
struct FinancialError(String);

impl fmt::Display for FinancialError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Financial calculation error: {}", self.0)
    }
}

impl Error for FinancialError {}

fn calculate_financials(rate: f64, params: &FinancialParams) -> Result<f64, Box<dyn Error>> {
    let mut cumulative_profit = 0.0;

    for year in 1..=params.num_years {
        let tsn = params.initial_tsn + params.au_hours * year as f64;
        let escalated_rate = rate * (1.0 + params.rate_escalation / 100.0).powi(year - 1);

        if !escalated_rate.is_finite() {
            return Err(Box::new(FinancialError("Escalated rate calculation overflow or NaN".into())));
        }

        let engine_revenue = params.au_hours * escalated_rate;
        let aic_revenue = engine_revenue * params.aic / 100.0;
        let total_revenue = engine_revenue + aic_revenue;

        if !total_revenue.is_finite() {
            return Err(Box::new(FinancialError("Revenue calculation overflow or NaN".into())));
        }

        let hsi = tsn >= params.hsi_tsn && (year == 1 || tsn - params.au_hours < params.hsi_tsn);
        let overhaul = tsn >= params.overhaul_tsn && (year == 1 || tsn - params.au_hours < params.overhaul_tsn);

        let hsi_cost = if hsi { params.hsi_cost } else { 0.0 };
        let overhaul_cost = if overhaul { params.overhaul_cost } else { 0.0 };
        let total_cost = hsi_cost + overhaul_cost;
        let total_profit = total_revenue - total_cost;
        cumulative_profit += total_profit;

        if !cumulative_profit.is_finite() {
            return Err(Box::new(FinancialError("Cumulative profit calculation overflow or NaN".into())));
        }
    }

    Ok(cumulative_profit)
}

fn newton_raphson<F, G>(
    f: F,
    df: G,
    mut x0: f64,
    xtol: f64,
    max_iter: usize,
) -> Result<(f64, usize), Box<dyn Error>>
where
    F: Fn(f64) -> Result<f64, Box<dyn Error>>,
    G: Fn(f64) -> Result<f64, Box<dyn Error>>,
{
    for i in 0..max_iter {
        let fx = f(x0)?;
        if fx.abs() < xtol {
            return Ok((x0, i + 1));
        }

        let dfx = df(x0)?;
        if dfx == 0.0 {
            return Err("Derivative is zero, can't proceed with Newton-Raphson".into());
        }

        x0 -= fx / dfx;
    }
    Err(format!("Newton-Raphson method did not converge within {} iterations", max_iter).into())
}

fn goal_seek(
    target_profit: f64,
    params: &FinancialParams,
    initial_guess: f64,
) -> Result<(f64, usize), Box<dyn Error>> {
    let objective = |rate: f64| -> Result<f64, Box<dyn Error>> {
        let profit = calculate_financials(rate, params)?;
        Ok(profit - target_profit)
    };

    let derivative = |rate: f64| -> Result<f64, Box<dyn Error>> {
        const EPSILON: f64 = 1e-6;
        let f1 = objective(rate + EPSILON)?;
        let f2 = objective(rate)?;
        Ok((f1 - f2) / EPSILON)
    };

    newton_raphson(objective, derivative, initial_guess, 1e-8, 100)
}

fn main() -> Result<(), Box<dyn Error>> {
    let params = FinancialParams {
        num_years: 10,
        au_hours: 450.0,
        initial_tsn: 100.0,
        rate_escalation: 5.0,
        aic: 10.0,
        hsi_tsn: 1000.0,
        overhaul_tsn: 3000.0,
        hsi_cost: 50000.0,
        overhaul_cost: 100000.0,
    };

    params.validate()?;

    let initial_rate = 320.0;
    let target_profit = 3_000_000.0;

    let start = Instant::now();

    let initial_cumulative_profit = calculate_financials(initial_rate, &params)?;

    println!("Initial Warranty Rate: {:.2}", initial_rate);
    println!("Initial Cumulative Profit: {:.2}", initial_cumulative_profit);

    let (optimal_rate, iterations) = goal_seek(target_profit, &params, initial_rate)?;

    println!("\nOptimal Warranty Rate to achieve {:.2} profit: {:.7}", target_profit, optimal_rate);
    println!("Number of iterations: {}", iterations);

    let final_cumulative_profit = calculate_financials(optimal_rate, &params)?;
    println!("\nFinal Cumulative Profit: {:.2}", final_cumulative_profit);

    let elapsed = start.elapsed();
    println!("\nExecution time: {:?}", elapsed);

    Ok(())
}
