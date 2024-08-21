# financialModel
finanacialModel


docker stop $(docker ps -aq) && \
docker rm $(docker ps -aq) && \
docker rmi $(docker images -q) && \
docker volume rm $(docker volume ls -q) && \
docker network prune -f && \
docker system prune -af --volumes

docker build -t financial_calculator .
docker run -d -p 8080:8080 financial_calculator
