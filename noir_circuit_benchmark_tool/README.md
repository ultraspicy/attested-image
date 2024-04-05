
# Experiment
## Data Generation
cd data_generation;
python generate.py;
will automatically save the data to circom/sp1/noir, and output the square sum

## SP1
### Installation
https://succinctlabs.github.io/sp1/getting-started/install.html
### Compilation
Build Project: cd program; cargo prove build
### Testing Speed
cd ../script
RUST_LOG=info cargo run --release

## Circom
### Installation
https://docs.circom.io/getting-started/installation/#installing-circom
### Running


## Noir
### Installation
https://noir-lang.org/docs/getting_started/installation/
### Presetting
In main.nr, setting the sum limit and the vector length
### Compilation
nargo check;
### Testing Speed
time nargo prove
time nargo verify