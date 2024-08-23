# Financial Model - Production Branch

## Prerequisites

- Docker

## Building and Running the Application

### Build the Docker Image:
```bash
docker build -t financial-calculator-go .
```

### Run the Docker Container:
```bash
docker run -d -p 8080:8080 financial-calculator-go
```

## API Usage

Send a POST request to `http://localhost:8080/calculate` with the following JSON body:

```json
{
  "numYears": 10,
  "auHours": 450,
  "initialTSN": 100,
  "rateEscalation": 5,
  "aic": 10,
  "hsitsn": 1000,
  "overhaulTSN": 3000,
  "hsiCost": 50000,
  "overhaulCost": 100000,
  "targetProfit": 3000000,
  "initialRate": 320
}
```

### Example Response:
```json
{
  "computationTime": 0.000953729,
  "finalCumulativeProfit": 2999999.9999999986,
  "initialCumulativeProfit": 1842338.1776309344,
  "initialWarrantyRate": 320,
  "iterations": 3,
  "optimalWarrantyRate": 505.93820432563325
}
```
### Postman:
![image](https://github.com/user-attachments/assets/7e0f7d05-1d08-456c-92dd-026ec708a82b)

## Managing Docker

### Stop All Containers:
```bash
docker stop $(docker ps -aq)
```

### Remove All Containers:
```bash
docker rm $(docker ps -aq)
```

### Remove All Images:
```bash
docker rmi $(docker images -q)
```

### Prune Networks:
```bash
docker network prune -f
```

**Note:**  
Adjust the host port in the `docker run` command if port 8080 is already in use on your machine.
