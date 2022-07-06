# Build docker image with testing program
docker build -f script/test_image/Dockerfile . -t $TESTING_IMAGE_NAME