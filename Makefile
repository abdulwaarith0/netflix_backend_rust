build:
	docker-compose build

up:
	docker-compose up

down:
	docker-compose down

clean:
	docker-compose down -v
	docker system prune --volumes -f

run:
	docker-compose up -d