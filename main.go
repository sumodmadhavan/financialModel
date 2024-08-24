// main.go
package main

import (
	"log"
	"net/http"

	"github.com/gin-gonic/gin"
)

// EndpointHandler type definition
type EndpointHandler = gin.HandlerFunc

// APIEndpoint struct definition
type APIEndpoint struct {
	Path    string
	Method  string
	Handler EndpointHandler
}

// APIServer struct definition
type APIServer struct {
	router    *gin.Engine
	endpoints []APIEndpoint
}

// NewAPIServer function
func NewAPIServer() *APIServer {
	return &APIServer{
		router:    gin.Default(),
		endpoints: []APIEndpoint{},
	}
}

// AddEndpoint method
func (s *APIServer) AddEndpoint(endpoint APIEndpoint) {
	s.endpoints = append(s.endpoints, endpoint)
}

// SetupRoutes method
func (s *APIServer) SetupRoutes() {
	for _, endpoint := range s.endpoints {
		switch endpoint.Method {
		case http.MethodGet:
			s.router.GET(endpoint.Path, endpoint.Handler)
		case http.MethodPost:
			s.router.POST(endpoint.Path, endpoint.Handler)
		}
	}
}

// Run method
func (s *APIServer) Run(addr string) error {
	return s.router.Run(addr)
}

func main() {
	server := NewAPIServer()

	server.AddEndpoint(APIEndpoint{
		Path:    "/goalseek",
		Method:  http.MethodPost,
		Handler: GoalSeekHandler, // This should be defined in goalseek.go
	})

	server.AddEndpoint(APIEndpoint{
		Path:    "/runout",
		Method:  http.MethodPost,
		Handler: RunoutHandler, // This should be defined in runout.go
	})

	server.SetupRoutes()

	log.Fatal(server.Run(":8080"))
}
