if [ $# -lt 2 ]; then
    echo "Usage: $0 <circom file name> <input file name> [--nodejs]"
    echo "[--nodejs] is optional, if not passed, the script will use Cpp to generate the witness"
    exit 1
fi

# define the base path for the circuit library
CIRCOMLIB_PATH=/home/${USER}/node_modules

# get the file name without the extension
filename=$(basename -- "$1")
CIRCOM_FILENAME=${filename%.*}

echo "Compiling circuit [${CIRCOM_FILENAME}] ..."

# Create the output directory if it doesn't exist
mkdir -p output/compiled_circuit > /dev/null
mkdir -p output/compiled_circuit/compiled_${CIRCOM_FILENAME} > /dev/null

# compile the circuits
circom ${1} --r1cs --wasm --sym --c --output output/compiled_circuit/compiled_${CIRCOM_FILENAME} -l ${CIRCOMLIB_PATH} -l ./circuits/base

# Generate the witness
# if [[ $* == *--nodejs* ]]; then
echo "Compiling with [NodeJS] ..."
cd output/compiled_circuit/compiled_${CIRCOM_FILENAME}/${CIRCOM_FILENAME}_js
node generate_witness.js ${CIRCOM_FILENAME}.wasm ../../../../../circuits/${CIRCOM_FILENAME}/${2} ../${CIRCOM_FILENAME}_witness.wtns
# else
#     echo "Compiling with [Cpp] ..."
#     cd output/compiled_circuit/compiled_${CIRCOM_FILENAME}/${CIRCOM_FILENAME}_cpp
#     make
    
#     ./${CIRCOM_FILENAME} ../../../../${2} ../${CIRCOM_FILENAME}_witness.wtns
# fi

echo "Witness generated [${CIRCOM_FILENAME}_witness.wtns]"