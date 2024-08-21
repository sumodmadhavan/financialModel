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
    fn validate(&self) -> Result<(), &'static str> {
        if self.num_years <= 0
           || self.au_hours <= 0.0
           || self.initial_tsn < 0.0
           || self.rate_escalation < 0.0
           || !(0.0..=100.0).contains(&self.aic)
           || self.hsi_tsn <= 0.0
           || self.overhaul_tsn <= 0.0
           || self.hsi_cost < 0.0
           || self.overhaul_cost < 0.0
        {
            return Err("Invalid parameter values");
        }
        Ok(())
    }
}

#[derive(Debug)]
struct FinancialError(&'static str);

impl fmt::Display for FinancialError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Financial calculation error: {}", self.0)
    }
}

impl Error for FinancialError {}

fn calculate_financials(rate: f64, params: &FinancialParams) -> Result<f64, FinancialError> {
    let mut cumulative_profit = 0.0;
    let escalation_factor = 1.0 + params.rate_escalation / 100.0;

    for year in 1..=params.num_years {
        let tsn = params.initial_tsn + params.au_hours * year as f64;
        let escalated_rate = rate * escalation_factor.powi(year - 1);

        if !escalated_rate.is_finite() {
            return Err(FinancialError("Escalated rate calculation overflow or NaN"));
        }

        let engine_revenue = params.au_hours * escalated_rate;
        let total_revenue = engine_revenue * (1.0 + params.aic / 100.0);

        if !total_revenue.is_finite() {
            return Err(FinancialError("Revenue calculation overflow or NaN"));
        }

        let hsi = tsn >= params.hsi_tsn && (year == 1 || tsn - params.au_hours < params.hsi_tsn);
        let overhaul = tsn >= params.overhaul_tsn && (year == 1 || tsn - params.au_hours < params.overhaul_tsn);

        let total_cost = if hsi { params.hsi_cost } else { 0.0 }
                       + if overhaul { params.overhaul_cost } else { 0.0 };

        cumulative_profit += total_revenue - total_cost;

        if !cumulative_profit.is_finite() {
            return Err(FinancialError("Cumulative profit calculation overflow or NaN"));
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
) -> Result<(f64, usize), FinancialError>
where
    F: Fn(f64) -> Result<f64, FinancialError>,
    G: Fn(f64) -> Result<f64, FinancialError>,
{
    for i in 0..max_iter {
        let fx = f(x0)?;
        if fx.abs() < xtol {
            return Ok((x0, i + 1));
        }

        let dfx = df(x0)?;
        if dfx == 0.0 {
            return Err(FinancialError("Derivative is zero, can't proceed with Newton-Raphson"));
        }

        x0 -= fx / dfx;
    }
    Err(FinancialError("Newton-Raphson method did not converge within specified iterations"))
}

fn goal_seek(
    target_profit: f64,
    params: &FinancialParams,
    initial_guess: f64,
) -> Result<(f64, usize), FinancialError> {
    let objective = |rate: f64| -> Result<f64, FinancialError> {
        let profit = calculate_financials(rate, params)?;
        Ok(profit - target_profit)
    };

    let derivative = |rate: f64| -> Result<f64, FinancialError> {
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
        App::new()
            .service(web::resource("/goal_seek").route(web::post().to(goal_seek_handler)))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
