package main

import (
	"math"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"gonum.org/v1/gonum/optimize"
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

func calculateFinancials(rate float64, params FinancialParams) float64 {
	years := make([]float64, params.NumYears)
	tsn := make([]float64, params.NumYears)
	escalatedRate := make([]float64, params.NumYears)
	totalRevenue := make([]float64, params.NumYears)
	totalCost := make([]float64, params.NumYears)
	totalProfit := make([]float64, params.NumYears)

	for i := 0; i < params.NumYears; i++ {
		years[i] = float64(i + 1)
		tsn[i] = params.InitialTSN + params.AuHours*years[i]
		escalatedRate[i] = rate * math.Pow(1+params.RateEscalation/100, years[i]-1)
		engineRevenue := params.AuHours * escalatedRate[i]
		aicRevenue := engineRevenue * params.AIC / 100
		totalRevenue[i] = engineRevenue + aicRevenue

		hsi := (tsn[i] >= params.HSITSN) && (i == 0 || tsn[i-1] < params.HSITSN)
		overhaul := (tsn[i] >= params.OverhaulTSN) && (i == 0 || tsn[i-1] < params.OverhaulTSN)

		if hsi {
			totalCost[i] += params.HSICost
		}
		if overhaul {
			totalCost[i] += params.OverhaulCost
		}

		totalProfit[i] = totalRevenue[i] - totalCost[i]
	}

	cumulativeProfit := 0.0
	for _, profit := range totalProfit {
		cumulativeProfit += profit
	}

	return cumulativeProfit
}
func goalSeek(targetProfit float64, params FinancialParams, tolerance float64) (float64, int, error) {
	problem := optimize.Problem{
		Func: func(x []float64) float64 {
			return math.Abs(calculateFinancials(x[0], params) - targetProfit)
		},
	}

	result, err := optimize.Minimize(problem, []float64{params.InitialRate}, &optimize.Settings{
		MajorIterations: 1000,
		Converger: &optimize.FunctionConverge{
			Absolute:   tolerance,
			Iterations: 100,
		},
	}, nil)

	if err != nil {
		return 0, 0, err
	}

	return result.X[0], int(result.Stats.MajorIterations), nil
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

		initialCumulativeProfit := calculateFinancials(params.InitialRate, params)

		optimalRate, iterations, err := goalSeek(params.TargetProfit, params, 1e-6)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}

		finalCumulativeProfit := calculateFinancials(optimalRate, params)

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
