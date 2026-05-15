abort(){
echo "$1"
exit
}
echo "What are you going to do?"
echo "1.Just clean"
echo "2.Just build"
echo "3.Build and copy to QSTools(main)"
echo "4.Build and copy+push to QSTools(main)"
echo "5.Clean and build"
echo "6.Clean and build and copy to QSTools(main)"
echo "7.Clean and build and copy+push to QSTools(main)"

read CleanInputs
case $CleanInputs in
    1)
    cargo clean
    ;;
    2)
    cargo build --release && echo "build successful" || abort "build failed"
    ;;
    3)
    cargo build --release && echo "build successful" || abort "build failed"
    cp -af ./target/release/blkops ./../main/binary/blkops
    ;;
    4)
    cargo build --release && echo "build successful" || abort "build failed"
    cp -af ./target/release/blkops ./../main/binary/blkops
    cd ../main
    git add binary/
    git commit -m "Update the blkops"
    git push
    ;;
    5)
    cargo clean
    cargo build --release && echo "build successful" || abort "build failed"
    ;;
    6)
    cargo clean
    cargo build --release && echo "build successful" || abort "build failed"
    cp -af ./target/release/blkops ./../main/binary/blkops
    ;;
    7)
    cargo clean
    cargo build --release && echo "build successful" || abort "build failed"
    cp -af ./target/release/blkops ./../main/binary/blkops
    cd ../main
    git add binary/
    git commit -m "Update the blkops"
    git push
    ;;
esac
