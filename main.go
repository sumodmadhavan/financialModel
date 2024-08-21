package main

import (
	"fmt"
	"math"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
)

type FinancialParams struct {
	NumYears       int     `json:"numYears"`
	AuHours        float64 `json:"auHours"`
	InitialTSN     float64 `json:"initialTSN"`
	RateEscalation float64 `json:"rateEscalation"`
	AIC            float64 `json:"aic"`
	HSITSN         float64 `json:"hsitsn"`
	OverhaulTSN    float64 `json:"overhaulTSN"`
	HSICost        float64 `json:"hsiCost"`
	OverhaulCost   float64 `json:"overhaulCost"`
	TargetProfit   float64 `json:"targetProfit"`
	InitialRate    float64 `json:"initialRate"`
}

func (p FinancialParams) Validate() error {
	if p.NumYears <= 0 {
		return fmt.Errorf("NumYears must be positive")
	}
	if p.AuHours <= 0 {
		return fmt.Errorf("AuHours must be positive")
	}
	if p.InitialTSN < 0 {
		return fmt.Errorf("InitialTSN cannot be negative")
	}
	if p.RateEscalation < 0 {
		return fmt.Errorf("RateEscalation cannot be negative")
	}
	if p.AIC < 0 || p.AIC > 100 {
		return fmt.Errorf("AIC must be between 0 and 100")
	}
	if p.HSITSN <= 0 {
		return fmt.Errorf("HSITSN must be positive")
	}
	if p.OverhaulTSN <= 0 {
		return fmt.Errorf("OverhaulTSN must be positive")
	}
	if p.HSICost < 0 {
		return fmt.Errorf("HSICost cannot be negative")
	}
	if p.OverhaulCost < 0 {
		return fmt.Errorf("OverhaulCost cannot be negative")
	}
	if p.TargetProfit <= 0 {
		return fmt.Errorf("TargetProfit must be positive")
	}
	if p.InitialRate <= 0 {
		return fmt.Errorf("InitialRate must be positive")
	}
	return nil
}

func calculateFinancials(rate float64, params FinancialParams) (float64, error) {
	var cumulativeProfit float64

	for year := 1; year <= params.NumYears; year++ {
		tsn := params.InitialTSN + params.AuHours*float64(year)
		escalatedRate := rate * math.Pow(1+params.RateEscalation/100, float64(year-1))

		if math.IsInf(escalatedRate, 0) || math.IsNaN(escalatedRate) {
			return 0, fmt.Errorf("escalated rate calculation overflow or NaN")
		}

		engineRevenue := params.AuHours * escalatedRate
		aicRevenue := engineRevenue * params.AIC / 100
		totalRevenue := engineRevenue + aicRevenue

		if math.IsInf(totalRevenue, 0) || math.IsNaN(totalRevenue) {
			return 0, fmt.Errorf("revenue calculation overflow or NaN")
		}

		hsi := tsn >= params.HSITSN && (year == 1 || tsn-params.AuHours < params.HSITSN)
		overhaul := tsn >= params.OverhaulTSN && (year == 1 || tsn-params.AuHours < params.OverhaulTSN)

		hsiCost := 0.0
		if hsi {
			hsiCost = params.HSICost
		}
		overhaulCost := 0.0
		if overhaul {
			overhaulCost = params.OverhaulCost
		}
		totalCost := hsiCost + overhaulCost
		totalProfit := totalRevenue - totalCost
		cumulativeProfit += totalProfit

		if math.IsInf(cumulativeProfit, 0) || math.IsNaN(cumulativeProfit) {
			return 0, fmt.Errorf("cumulative profit calculation overflow or NaN")
		}
	}

	return cumulativeProfit, nil
}

func newtonRaphson(f, df func(float64) (float64, error), x0, xtol float64, maxIter int) (float64, int, error) {
	for i := 0; i < maxIter; i++ {
		fx, err := f(x0)
		if err != nil {
			return 0, i, err
		}
		if math.Abs(fx) < xtol {
			return x0, i + 1, nil // Return the iteration count
		}

		dfx, err := df(x0)
		if err != nil {
			return 0, i, err
		}
		if dfx == 0 {
			return 0, i, fmt.Errorf("derivative is zero, can't proceed with Newton-Raphson")
		}

		x0 = x0 - fx/dfx
	}
	return 0, maxIter, fmt.Errorf("Newton-Raphson method did not converge within %d iterations", maxIter)
}

func goalSeek(targetProfit float64, params FinancialParams, initialGuess float64) (float64, int, error) {
	objective := func(rate float64) (float64, error) {
		profit, err := calculateFinancials(rate, params)
		if err != nil {
			return 0, err
		}
		return profit - targetProfit, nil
	}

	derivative := func(rate float64) (float64, error) {
		epsilon := 1e-6 // small value for numerical derivative approximation
		f1, err1 := objective(rate + epsilon)
		f2, err2 := objective(rate)
		if err1 != nil || err2 != nil {
			return 0, fmt.Errorf("error calculating derivative")
		}
		return (f1 - f2) / epsilon, nil
	}

	return newtonRaphson(objective, derivative, initialGuess, 1e-8, 100)
}

func main() {
	r := gin.Default()

	r.POST("/calculate", func(c *gin.Context) {
		startTime := time.Now()

		var params FinancialParams
		if err := c.ShouldBindJSON(&params); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
			return
		}

		if err := params.Validate(); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
			return
		}

		initialCumulativeProfit, err := calculateFinancials(params.InitialRate, params)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}

		optimalRate, iterations, err := goalSeek(params.TargetProfit, params, params.InitialRate)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}

		finalCumulativeProfit, err := calculateFinancials(optimalRate, params)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}

		duration := time.Since(startTime)

		c.JSON(http.StatusOK, gin.H{
			"initialWarrantyRate":     params.InitialRate,
			"initialCumulativeProfit": initialCumulativeProfit,
			"optimalWarrantyRate":     optimalRate,
			"iterations":              iterations,
			"finalCumulativeProfit":   finalCumulativeProfit,
			"computationTime":         duration.Seconds(),
		})
	})

	r.Run(":8080")
}
