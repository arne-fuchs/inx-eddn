use ethers::prelude::Abigen;

fn main() {
    Abigen::new("EDCAS", "./contracts/EDCAS.abi").unwrap().generate().unwrap().write_to_file("./src/edcas_contract.rs").unwrap();
}