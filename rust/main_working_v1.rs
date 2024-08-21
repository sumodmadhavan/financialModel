use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::time::Instant;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
struct GoalSeekRequest {
    target_profit: f64,
    params: FinancialParams,
    initial_guess: f64,
}

#[derive(Serialize)]
struct GoalSeekResponse {
    optimal_rate: f64,
    iterations: usize,
    final_cumulative_profit: f64,
    execution_time_ms: f64
}

async fn goal_seek_handler(req: web::Json<GoalSeekRequest>) -> impl Responder {
    let start_time = Instant::now();

    if let Err(e) = req.params.validate() {
        return HttpResponse::BadRequest().body(format!("Invalid parameters: {}", e));
    }
    let result = goal_seek(req.target_profit, &req.params, req.initial_guess);

    match result {
        Ok((optimal_rate, iterations)) => {
            match calculate_financials(optimal_rate, &req.params) {
                Ok(final_cumulative_profit) => {
                    let execution_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
                    let response = GoalSeekResponse {
                        optimal_rate,
                        iterations,
                        final_cumulative_profit,
                        execution_time_ms,
                    };
                    HttpResponse::Ok().json(response)
                }
                Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
            }
        }
        Err(e) => HttpResponse::BadRequest().body(format!("Error: {}", e)),
    }
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://0.0.0.0:8080");
    HttpServer::new(|| {
        App::new().service(web::resource("/goal_seek").route(web::post().to(goal_seek_handler)))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
