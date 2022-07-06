# Set environment variables.
export SERVER_IMAGE_NAME=$(cat config.json | jq -r '.server_image_name')
export EXERCISES_PATH=$(cat config.json | jq -r '.data_path')
export SERVER_CONTAINER_NAME=$(cat config.json | jq -r '.server_container_name')

# Build docker image with compiled server.
docker build . -t $SERVER_IMAGE_NAME

# Make sure that container with provided name doesn't exist.
docker container rm "$SERVER_CONTAINER_NAME"

# Create docker container.
docker container create \
    -v "$EXERCISES_PATH/exercises:/exercises" \
    -v /var/run/docker.sock:/var/run/docker.sock \
    --name "$SERVER_CONTAINER_NAME" \
    "$SERVER_IMAGE_NAME"