# Set environment variable.
export TESTING_IMAGE_NAME=$(cat config.json | jq -r '.testing_image_name')

# Compile server.
cargo build --release
