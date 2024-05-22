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

# Create the output directory if it doesn't exist
mkdir -p output/compiled_circuit > /dev/null
mkdir -p output/compiled_circuit/compiled_${CIRCOM_FILENAME} > /dev/null

# compile the circuits
echo "==================== Compiling circuit ${CIRCOM_FILENAME} ... ====================" >&2
/usr/bin/time circom ${1} --r1cs --wasm --sym --c --output output/compiled_circuit/compiled_${CIRCOM_FILENAME} -l ${CIRCOMLIB_PATH} -l ./circuits/base
#echo "Circuit compiled in output/compiled_circuit/compiled_${CIRCOM_FILENAME}"

# Generate the witness
echo "==================== Generating witness ...  ====================" >&2
cd output/compiled_circuit/compiled_${CIRCOM_FILENAME}/${CIRCOM_FILENAME}_js
/usr/bin/time node generate_witness.js ${CIRCOM_FILENAME}.wasm ../../../../../circuits/${CIRCOM_FILENAME}/${2} ${CIRCOM_FILENAME}_witness.wtns

#echo "Witness generated [${CIRCOM_FILENAME}_witness.wtns]"

# # start a new "powers of tau" ceremony
# echo "==================== Start a new "powers of tau" ceremony: ...  ====================" >&2
# cd ..
# /usr/bin/time snarkjs powersoftau new bn128 12 pot12_0000.ptau -v

# echo "==================== contribute to the ceremony: ...  ====================" >&2
# /usr/bin/time echo "cs251" | snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="First contribution" -v

# echo "==================== PHASE2 ...  ====================" >&2
# /usr/bin/time snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau -v

echo "==================== generate verification file  ...  ====================" >&2
/usr/bin/time snarkjs groth16 setup ${CIRCOM_FILENAME}.r1cs pot12_final.ptau ${CIRCOM_FILENAME}_0000.zkey

echo ""==================== Contribute to the phase 2 of the ceremony ... "====================" >&2
/usr/bin/time echo "cs251" | snarkjs zkey contribute ${CIRCOM_FILENAME}_0000.zkey ${CIRCOM_FILENAME}_0001.zkey --name="cs251" -v

echo "==================== Export the verification key ====================" >&2
/usr/bin/time snarkjs zkey export verificationkey ${CIRCOM_FILENAME}_0001.zkey verification_key.json

echo "==================== Generating a Proof ====================" >&2
pwd
/usr/bin/time snarkjs groth16 prove ${CIRCOM_FILENAME}_0001.zkey ./${CIRCOM_FILENAME}_js/${CIRCOM_FILENAME}_witness.wtns proof.json public.json

echo "==================== Verifying a Proof ====================" >&2
/usr/bin/time snarkjs groth16 verify verification_key.json public.json proof.json
