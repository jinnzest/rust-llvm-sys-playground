 function pause(){
   read -p "$*"
}

file="./build.properties"

if [ -f "$file" ]
then
    echo "$file found."
 . $file

LLVM_SYS_70_PREFIX=$llvm_path cargo run --verbose && printf "\nRunning generated executable...\n\n" &&DYLD_LIBRARY_PATH=../test-lib/target/debug  ./target/out
else
    echo "$file not found."
fi
