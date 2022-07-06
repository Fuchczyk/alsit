# Set environment variables.
export SERVER_IMAGE_NAME=$(cat config.json | jq -r '.server_image_name')
export EXERCISES_PATH=$(cat config.json | jq -r '.data_path')
export SERVER_CONTAINER_NAME=$(cat config.json | jq -r '.server_container_name')

# Build docker image with compiled server.
docker image build . -t $SERVER_IMAGE_NAME