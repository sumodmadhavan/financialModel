package main

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
)

func TestGoalSeekEndpoint(t *testing.T) {
	router := setupRouter()

	validParams := FinancialParams{
		NumYears:       10,
		AuHours:        450,
		InitialTSN:     100,
		RateEscalation: 5,
		AIC:            10,
		HSITSN:         1000,
		OverhaulTSN:    3000,
		HSICost:        50000,
		OverhaulCost:   100000,
		TargetProfit:   3000000,
		InitialRate:    320,
	}

	tests := []struct {
		name           string
		payload        FinancialParams
		expectedStatus int
		expectedBody   map[string]interface{}
	}{
		{
			name:           "Valid input - original case",
			payload:        validParams,
			expectedStatus: http.StatusOK,
			expectedBody: map[string]interface{}{
				"optimalWarrantyRate": 505.93820432563325,
				"iterations":          float64(3),
			},
		},
		{
			name: "Invalid input - zero NumYears",
			payload: func() FinancialParams {
				p := validParams
				p.NumYears = 0
				return p
			}(),
			expectedStatus: http.StatusBadRequest,
			expectedBody: map[string]interface{}{
				"error": "NumYears must be positive",
			},
		},
		{
			name: "Invalid input - zero TargetProfit",
			payload: func() FinancialParams {
				p := validParams
				p.TargetProfit = 0
				return p
			}(),
			expectedStatus: http.StatusBadRequest,
			expectedBody: map[string]interface{}{
				"error": "TargetProfit must be positive",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			jsonPayload, _ := json.Marshal(tt.payload)
			req, _ := http.NewRequest("POST", "/goalseek", bytes.NewBuffer(jsonPayload))
			req.Header.Set("Content-Type", "application/json")
			w := httptest.NewRecorder()
			router.ServeHTTP(w, req)

			assert.Equal(t, tt.expectedStatus, w.Code)

			var response map[string]interface{}
			err := json.Unmarshal(w.Body.Bytes(), &response)
			assert.NoError(t, err)

			if tt.expectedStatus == http.StatusOK {
				assert.InDelta(t, tt.expectedBody["optimalWarrantyRate"], response["optimalWarrantyRate"], 1e-6)
				assert.Equal(t, tt.expectedBody["iterations"], response["iterations"])
			} else {
				assert.Equal(t, tt.expectedBody["error"], response["error"])
			}
		})
	}
}

func setupRouter() *gin.Engine {
	router := gin.Default()
	router.POST("/goalseek", GoalSeekHandler)
	return router
}
