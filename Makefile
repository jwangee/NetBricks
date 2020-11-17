all: netbricks

netbricks:
	./build.sh

docker: netbricks
	sudo docker build -t levaitamas/netbricks --no-cache .

docker-clean:
	sudo docker rm --force $(docker ps -a -q)
