#!/bin/bash

# start a new "powers of tau" ceremony, the constraint should be less than or equal to 2**16
echo "==================== Start a new "powers of tau" ceremony ...  ====================" >&2
cd ../circuit_params
/usr/bin/time snarkjs powersoftau new bn128 12 pot12_0000.ptau -v

echo "==================== contribute to the ceremony ...  ====================" >&2
/usr/bin/time snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="First contribution" -v -entropy="cs251"

echo "==================== PHASE2 ...  ====================" >&2
/usr/bin/time snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau -v
