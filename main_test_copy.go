package main

import (
	"math"
	"testing"
)

func TestCalculateFinancialsOptimalRateOnly(t *testing.T) {
	tests := []struct {
		name         string
		numYears     int
		initialRate  float64
		targetProfit float64
		wantRate     float64
		wantErr      bool
	}{
		{
			name:         "20 years, rate 250, target profit 2,000,000",
			numYears:     20,
			initialRate:  250,
			targetProfit: 2000000,
			wantRate:     131.35669183835628,
			wantErr:      false,
		},
		{
			name:         "35 years, rate 320, target profit 3,000,000",
			numYears:     35,
			initialRate:  320,
			targetProfit: 3000000,
			wantRate:     70.45631874177171,
			wantErr:      false,
		},
		{
			name:         "Validation error when initialRate and targetProfit are zero",
			numYears:     20,
			initialRate:  0,
			targetProfit: 0,
			wantRate:     0, // No need to check the rate as we expect an error
			wantErr:      true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			params := FinancialParams{
				NumYears:       tt.numYears,
				AuHours:        450,    // Hardcoded
				InitialTSN:     100,    // Hardcoded
				RateEscalation: 5,      // Hardcoded
				AIC:            10,     // Hardcoded
				HSITSN:         1000,   // Hardcoded
				OverhaulTSN:    3000,   // Hardcoded
				HSICost:        50000,  // Hardcoded
				OverhaulCost:   100000, // Hardcoded
				TargetProfit:   tt.targetProfit,
				InitialRate:    tt.initialRate,
			}

			gotOptimalRate, _, err := goalSeek(params.TargetProfit, params, params.InitialRate)
			if (err != nil) != tt.wantErr {
				t.Errorf("goalSeek() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !tt.wantErr && math.Abs(gotOptimalRate-tt.wantRate) > 1e-6 {
				t.Errorf("goalSeek() optimalRate = %v, want %v", gotOptimalRate, tt.wantRate)
			}
		})
	}
}
