# Change workdir to root of the crate.
cd "$(dirname "$0")"
cd ..
cd ..

# Image name
imageName=$(cat config.json | jq -r '.testing_image_name')

# Build docker image with testing program
docker build -f script/test_image/Dockerfile . -t $imageName