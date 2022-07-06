# Set environment variables.
export TESTING_IMAGE_NAME=$(cat config.json | jq -r '.testing_image_name')

# Build docker image for testing.
source script/test_image/build_image.sh

# Run build
cargo build --release