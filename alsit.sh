# Change workdir to root of the crate (alsit.sh path).
cd "$(dirname "$0")"

runType=$1

case $runType in
    build)
        source script/build/image.sh
        ;;
    
    run)
        source script/build/build_project.sh
        ;;
    
    *)
        echo "Provided run option is not recognised."
        ;;
esac