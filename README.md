docker build -t financial-calculator-go .

docker run -d -p 8080:8080 financial-calculator-go

docker ps

curl -X POST -H "Content-Type: application/json" -d '{
  "initialRate": 320,
  "targetProfit": 3000000,
  "numYears": 10,
  "auHours": 450,
  "initialTSN": 100,
  "rateEscalation": 5,
  "aic": 10,
  "hsiTSN": 1000,
  "overhaulTSN": 3000,
  "hsiCost": 50000,
  "overhaulCost": 100000
}' http://localhost:8080/calculate
