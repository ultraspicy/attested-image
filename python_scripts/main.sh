if [ $# -lt 2 ]; then
    echo "Usage: $0 <circom file name> <input file name> [--nodejs]"
    echo "[--nodejs] is optional, if not passed, the script will use Cpp to generate the witness"
    exit 1
fi

# define the base path for the circuit library
CIRCOMLIB_PATH=/home/${USER}/node_modules
CIRCOM_CIRCUIT_ROOT=~/Developer/attested-image/circom_circuit_benchmark_tool
CIRCOM_SCRIPT_PATH=${CIRCOM_CIRCUIT_ROOT}/scripts
CIRCOM_OUTPUT_PATH=${CIRCOM_CIRCUIT_ROOT}/output
CIRCOM_INPUT_PAATH=~/Developer/attested-image/python_scripts/circom_circuit_input
CIRCUIT_PRRAM_ROOT=~/Developer/attested-image/circuit_params

# get the file name without the extension
filename=$(basename -- "$1")
CIRCOM_FILENAME=${filename%.*}

# echo ${CIRCOM_OUTPUT_PATH}

cd ${CIRCOM_CIRCUIT_ROOT}

# Create the output directory if it doesn't exist
mkdir -p ${CIRCOM_OUTPUT_PATH}/compiled_circuit > /dev/null
mkdir -p ${CIRCOM_OUTPUT_PATH}/compiled_circuit/compiled_${CIRCOM_FILENAME} > /dev/null

# # compile the circuits
# echo "==================== Compiling circuit ${CIRCOM_FILENAME} ... ====================" >&2
# /usr/bin/time circom ${CIRCOM_CIRCUIT_ROOT}/circuits/${CIRCOM_FILENAME}/${1} --r1cs --wasm --sym --c --output output/compiled_circuit/compiled_${CIRCOM_FILENAME} -l ${CIRCOMLIB_PATH} -l ./circuits/base
# #echo "Circuit compiled in output/compiled_circuit/compiled_${CIRCOM_FILENAME}"

# Generate the witness
echo "==================== Generating witness ...  ====================" >&2
cd ${CIRCOM_CIRCUIT_ROOT}/output/compiled_circuit/compiled_${CIRCOM_FILENAME}/${CIRCOM_FILENAME}_js
/usr/bin/time node generate_witness.js ${CIRCOM_FILENAME}.wasm ${CIRCOM_INPUT_PAATH}/${2} ${CIRCOM_FILENAME}_witness.wtns

# verification
cd ..
echo "==================== generate verification file  ...  ====================" >&2
/usr/bin/time snarkjs groth16 setup ${CIRCOM_FILENAME}.r1cs ${CIRCUIT_PRRAM_ROOT}/pot18_final.ptau ${CIRCOM_FILENAME}_0000.zkey

echo ""==================== Contribute to the phase 2 of the ceremony ... "====================" >&2
/usr/bin/time snarkjs zkey contribute ${CIRCOM_FILENAME}_0000.zkey ${CIRCOM_FILENAME}_0001.zkey --name="cs251" -v -entropy="cs251"

echo "==================== Export the verification key ====================" >&2
/usr/bin/time snarkjs zkey export verificationkey ${CIRCOM_FILENAME}_0001.zkey verification_key.json

echo "==================== Generating a Proof ====================" >&2
pwd
/usr/bin/time snarkjs groth16 prove ${CIRCOM_FILENAME}_0001.zkey ./${CIRCOM_FILENAME}_js/${CIRCOM_FILENAME}_witness.wtns proof.json public.json

echo "==================== Verifying a Proof ====================" >&2
/usr/bin/time snarkjs groth16 verify verification_key.json public.json proof.json
