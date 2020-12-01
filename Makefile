all: netbricks

netbricks:
	./build.sh

docker: netbricks
	sudo docker build -t ch8728847/netbricks:latest --no-cache .

docker-clean:
	sudo docker rm --force $(docker ps -a -q)
