## Compile the circuit
Compile the circuit to get a system of arithmetic equations representing it. As a result of the compilation we will also obtain programs to compute the witness. We can compile the circuit with the following command. cd into the folder containing the circom file 
```
(base) ~/Developer/attested-image/circom_circuit_benchmark_tool/circuits/diff_square_sum (main ✘)✭ ᐅ circom diff_square_sum.circom --r1cs --wasm --sym --c
template instances: 4
non-linear constraints: 3
linear constraints: 0
public inputs: 0
private inputs: 6
public outputs: 1
wires: 10
labels: 33
Written successfully: ./diff_square_sum.r1cs
Written successfully: ./diff_square_sum.sym
Written successfully: ./diff_square_sum_cpp/diff_square_sum.cpp and ./diff_square_sum_cpp/diff_square_sum.dat
Written successfully: ./diff_square_sum_cpp/main.cpp, circom.hpp, calcwit.hpp, calcwit.cpp, fr.hpp, fr.cpp, fr.asm and Makefile
Written successfully: ./diff_square_sum_js/diff_square_sum.wasm
Everything went okay
```

## Compile the witness
Using input.json as input file, then cd into the `<circuit_name>_js` directory under the circuit you work on
```
node generate_witness.js diff_square_sum.wasm ./../input.json witness.wtns
```

## Proving Circuits
Upon now, we have `.wtns` that contains all the computed signals, and `.r1cs` file that contains the constraints describing the circuit 

First, we start a new "powers of tau" ceremony:
```
(base) ~/Developer/attested-image/circom_circuit_benchmark_tool/circuits/diff_square_sum (main ✘)✭ ᐅ snarkjs powersoftau new bn128 12 pot12_0000.ptau -v
[DEBUG] snarkJS: Calculating First Challenge Hash
[DEBUG] snarkJS: Calculate Initial Hash: tauG1
[DEBUG] snarkJS: Calculate Initial Hash: tauG2
[DEBUG] snarkJS: Calculate Initial Hash: alphaTauG1
[DEBUG] snarkJS: Calculate Initial Hash: betaTauG1
[DEBUG] snarkJS: Blank Contribution Hash:
		786a02f7 42015903 c6c6fd85 2552d272
		912f4740 e1584761 8a86e217 f71f5419
		d25e1031 afee5853 13896444 934eb04b
		903a685b 1448b755 d56f701a fe9be2ce
[INFO]  snarkJS: First Contribution Hash:
		9e63a5f6 2b96538d aaed2372 481920d1
		a40b9195 9ea38ef9 f5f6a303 3b886516
		0710d067 c09d0961 5f928ea5 17bcdf49
		ad75abd2 c8340b40 0e3b18e9 68b4ffef
```
 we contribute to the ceremony:
 ```
 (base) ~/Developer/attested-image/circom_circuit_benchmark_tool/circuits/diff_square_sum (main ✘)✭ ᐅ snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="First contribution" -v
Enter a random text. (Entropy): cs251
[DEBUG] snarkJS: Calculating First Challenge Hash
[DEBUG] snarkJS: Calculate Initial Hash: tauG1
[DEBUG] snarkJS: Calculate Initial Hash: tauG2
[DEBUG] snarkJS: Calculate Initial Hash: alphaTauG1
[DEBUG] snarkJS: Calculate Initial Hash: betaTauG1
[DEBUG] snarkJS: processing: tauG1: 0/8191
[DEBUG] snarkJS: processing: tauG2: 0/4096
[DEBUG] snarkJS: processing: alphaTauG1: 0/4096
[DEBUG] snarkJS: processing: betaTauG1: 0/4096
[DEBUG] snarkJS: processing: betaTauG2: 0/1
[INFO]  snarkJS: Contribution Response Hash imported:
		f2962c19 09de6d43 06f12000 b3792fac
		ef92671a 7544bb78 a44b343b 4a3661d3
		a34c0e71 1bd5ece2 18399819 25579ae6
		22d409fb 658f66b3 8307d309 cacacb77
[INFO]  snarkJS: Next Challenge Hash:
		9bb79e1a fa9de072 6b5666b2 6a424d74
		8a988a1c 3ab1d5d0 2ad218c0 0b9e40ce
		b1812274 0cda922f dbcfe6bd 8509f3be
		ce977fbd f30f0f08 2d330030 38679399
```

Then PHASE2 
```
(base) ~/Developer/attested-image/circom_circuit_benchmark_tool/circuits/diff_square_sum (main ✘)✭ ᐅ snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau -v
[DEBUG] snarkJS: Starting section: tauG1
[DEBUG] snarkJS: tauG1: fft 0 mix start: 0/1
...
...
...
[DEBUG] snarkJS: betaTauG1: fft 12 join  12/12  1/1 3/4
```

Next, we generate a .zkey file that will contain the proving and verification keys together with all phase 2 contributions. Execute the following command to start a new zkey:
```
(base) ~/Developer/attested-image/circom_circuit_benchmark_tool/circuits/diff_square_sum (main ✘)✭ ᐅ snarkjs groth16 setup diff_square_sum.r1cs pot12_final.ptau diff_square_sum_0000.zkey
[INFO]  snarkJS: Reading r1cs
[INFO]  snarkJS: Reading tauG1
[INFO]  snarkJS: Reading tauG2
[INFO]  snarkJS: Reading alphatauG1
[INFO]  snarkJS: Reading betatauG1
[INFO]  snarkJS: Circuit hash:
		5fda59bf 05ff83b7 32ea4ca6 74ed7484
		bc6f33c4 2df44526 d3c8059d 44677881
		baa710a8 3e61a94d d23af018 b1260b37
		ffe53857 2d1457ed c55205fa 3e263e58
```
Contribute to the phase 2 of the ceremony:
```
(base) ~/Developer/attested-image/circom_circuit_benchmark_tool/circuits/diff_square_sum (main ✘)✭ ᐅ snarkjs zkey contribute diff_square_sum_0000.zkey diff_square_sum_0001.zkey --name="cs251" -v
Enter a random text. (Entropy): cs251
[DEBUG] snarkJS: Applying key: L Section: 0/8
[DEBUG] snarkJS: Applying key: H Section: 0/8
[INFO]  snarkJS: Circuit Hash:
		5fda59bf 05ff83b7 32ea4ca6 74ed7484
		bc6f33c4 2df44526 d3c8059d 44677881
		baa710a8 3e61a94d d23af018 b1260b37
		ffe53857 2d1457ed c55205fa 3e263e58
[INFO]  snarkJS: Contribution Hash:
		79c32224 0cea3527 99357048 ff2244cf
		a23edcb2 ee72778b dc20d6df 7433161f
		bbf53afe 10ddcef7 e263c8e7 44b017b8
		f8221f65 8f77d5f1 1565f52d 6778ac50
```
Export the verification key:
```
(base) ~/Developer/attested-image/circom_circuit_benchmark_tool/circuits/diff_square_sum (main ✘)✭ ᐅ snarkjs zkey export verificationkey diff_square_sum_0001.zkey verification_key.json
[INFO]  snarkJS: EXPORT VERIFICATION KEY STARTED
[INFO]  snarkJS: > Detected protocol: groth16
[INFO]  snarkJS: EXPORT VERIFICATION KEY FINISHED
```

## Generating a Proof
```
(base) ~/Developer/attested-image/circom_circuit_benchmark_tool/circuits/diff_square_sum (main ✘)✭ ᐅ snarkjs groth16 prove diff_square_sum_0001.zkey ./diff_square_sum_js/witness.wtns proof.json public.json
```
## Verifying a Proof
```
(base) ~/Developer/attested-image/circom_circuit_benchmark_tool/circuits/diff_square_sum (main ✘)✭ ᐅ snarkjs groth16 verify verification_key.json public.json proof.json
[INFO]  snarkJS: OK!
```