// goalseek.go
package main

import (
	"net/http"

	"github.com/gin-gonic/gin"
)

func GoalSeekHandler(c *gin.Context) {
	var params FinancialParams
	if err := c.ShouldBindJSON(&params); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := params.Validate(); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	optimalRate, iterations, err := goalSeek(params.TargetProfit, params, params.InitialRate)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"optimalWarrantyRate": optimalRate,
		"iterations":          iterations,
	})
}
