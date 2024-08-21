# Start from the official Go image
FROM golang:1.20-alpine

# Set the working directory inside the container
WORKDIR /app

# Copy go mod file
COPY go.mod ./

# Download all dependencies and verify
RUN go mod download && go mod verify

# Copy the source code into the container
COPY . .

# Build the application
RUN go build -o main .

# Expose port 8080 to the outside world
EXPOSE 8080

# Command to run the executable
CMD ["./main"]
