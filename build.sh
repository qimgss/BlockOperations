if [ -f ./Cargo.lock ] || [ -d ./target ]; then
    echo "What are you going to do?"
    echo "1.Just clean"
    echo "2.Just build"
    echo "3.Clean and build"
    read CleanInputs
    case $CleanInputs in
        1)
        cargo clean
        ;;
        2)
        cargo build --release && echo "build successful" || echo "build failed"
        ;;
        3)
        cargo clean
        cargo build --release && echo "build successful" || echo "build failed"
        ;;
    esac
else
    cargo build --release && echo "build successful" || echo "build failed"
fi