# Change workdir to root of the crate.
cd "$(dirname "$0")"
cd ..
cd ..

# Build docker image for testing.
sh script/test_image/build_image.sh