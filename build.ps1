docker build -t build-4-raspberrypi .
docker run --rm -w /app -v .:/app build-4-raspberrypi
