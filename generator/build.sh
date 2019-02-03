 function pause(){
   read -p "$*"
}

file="./build.properties"

if [ -f "$file" ]
then
    echo "$file found."
 . $file

LLVM_SYS_70_PREFIX=$llvm_path cargo build --verbose
else
    echo "$file not found."
fi
