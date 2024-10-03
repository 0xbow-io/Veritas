// Modified version of Arkworks-rs circom-compat witness calculation code

use fnv::FnvHasher;
use num::ToPrimitive;
use num_bigint::BigInt;
use num_traits::Zero;
use program_structure::{error_code::ReportCode, error_definition::Report};
use std::{collections::HashMap, hash::Hasher};
use wasmer::{
    imports, AsEngineRef, AsStoreMut, Exports, Function, Memory, MemoryType, Module, RuntimeError,
    Store, StoreMut, Value,
};
use wasmer_wasix::WasiEnv;

#[derive(thiserror::Error, Debug, Clone, Copy)]
#[error("{0}")]
struct ExitCode(u32);

pub struct WitnessCalculator {
    store: Store,
    exports: Exports,
    pub n64: u32,
    pub circom_version: u32,
    pub prime: BigInt,
}

impl Default for WitnessCalculator {
    fn default() -> Self {
        WitnessCalculator {
            store: Store::default(),
            exports: Exports::default(),
            n64: 0,
            circom_version: 0,
            prime: BigInt::default(),
        }
    }
}

impl WitnessCalculator {
    pub fn load(&mut self, wasm: impl AsRef<[u8]>) -> Result<(), Report> {
        let module = Module::new(&self.get_store().as_engine_ref(), wasm);
        match module {
            Ok(_) => {}
            Err(e) => {
                let mut err = Report::error(e.to_string(), ReportCode::FailedToLoadModule);
                err.add_note(format!(
                    "Was not able to load the witness calculation wasm module"
                ));
                return Err(err);
            }
        }
        let module = module.unwrap();
        let mut wasi_env_builder = WasiEnv::builder("calculateWitness");

        let memory = Memory::new(
            &mut self.get_mut_store(),
            MemoryType::new(2000, None, false),
        );
        match memory {
            Ok(_) => {}
            Err(e) => {
                let mut err = Report::error(e.to_string(), ReportCode::FailedToLoadModule);
                err.add_note(format!(
                    "Was not able to create the memory for the witness calculation wasm module"
                ));
                return Err(err);
            }
        }

        let imports = imports! {
            "env" => {
                "memory" => memory.unwrap(),
            },
            // Host function callbacks from the WASM
            "runtime" => {
                "exceptionHandler" => exception_handler(&mut self.store),
                "showSharedRWMemory" => show_memory(&mut self.store),
                "printErrorMessage" => print_error_message(&mut self.store),
                "writeBufferMessage" => write_buffer_message(&mut self.store),
            }
        };
        wasi_env_builder.add_imports(imports.into_iter());
        match wasi_env_builder.instantiate(module, &mut self.store) {
            Ok((instance, _)) => {
                self.exports.clone_from(&instance.exports);
            }
            Err(e) => {
                let mut err = Report::error(e.to_string(), ReportCode::FailedToLoadModule);
                err.add_note(format!(
                    "Was not able to instantiate the witness calculation wasm module"
                ));
                return Err(err);
            }
        }

        let n32 = self.get_field_num_len32();
        match n32 {
            Ok(n32) => {
                match self.get_raw_prime() {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
                let mut arr = vec![0; n32 as usize];
                for i in 0..n32 {
                    match self.read_shared_rw_memory(i) {
                        Ok(v) => {
                            arr[(n32 as usize) - (i as usize) - 1].clone_from(&v);
                        }
                        Err(e) => return Err(e),
                    }
                }
                let prime = from_array32(arr);
                self.n64 = ((prime.bits() - 1) / 64 + 1) as u32;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn calculate_witness<I: IntoIterator<Item = (String, Vec<BigInt>)>>(
        &mut self,
        inputs: I,
    ) -> Result<Vec<BigInt>, Report> {
        let mut w = Vec::new();

        match self.init(true) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
        let n32 = self.get_field_num_len32();

        match n32 {
            Ok(n32) => {
                for (name, values) in inputs.into_iter() {
                    let (msb, lsb) = fnv(&name);
                    for (i, value) in values.into_iter().enumerate() {
                        let f_arr = to_array32(&value, n32 as usize);
                        for j in 0..n32 {
                            match self
                                .write_shared_rw_memory(j, f_arr[(n32 as usize) - 1 - (j as usize)])
                            {
                                Ok(_) => (),
                                Err(e) => return Err(e),
                            }
                        }
                        match self.set_input_signal(msb, lsb, i as u32) {
                            Ok(_) => (),
                            Err(e) => return Err(e),
                        }
                    }
                }
                match self.get_witness_size() {
                    Ok(witness_size) => {
                        for i in 0..witness_size {
                            self.get_witness(i)?;
                            let mut arr = vec![0; n32 as usize];
                            for j in 0..n32 {
                                match self.read_shared_rw_memory(j) {
                                    Ok(v) => {
                                        arr[(n32 as usize) - 1 - (j as usize)] = v;
                                    }
                                    Err(e) => return Err(e),
                                }
                            }
                            w.push(from_array32(arr));
                        }
                        Ok(w)
                    }
                    Err(e) => return Err(e),
                }
            }
            Err(e) => return Err(e),
        }
    }

    fn get_mut_store(&mut self) -> StoreMut {
        self.store.as_store_mut()
    }
    fn get_store(&self) -> &Store {
        &self.store
    }
}

pub trait WitnessFunctions {
    fn init(&mut self, sanity_check: bool) -> Result<(), Report>;

    fn func(&mut self, name: &str) -> Result<&Function, Report>;
    fn do_call(&mut self, name: &str, params: &[Value]) -> Result<Box<[Value]>, Report>;
    fn get_u32(&mut self, name: &str, params: &[Value]) -> Result<u32, Report>;

    fn get_n_vars(&mut self) -> Result<u32, Report>;

    fn get_version(&mut self) -> Result<u32, Report>;
    fn get_field_num_len32(&mut self) -> Result<u32, Report>;
    fn get_raw_prime(&mut self) -> Result<(), Report>;
    fn read_shared_rw_memory(&mut self, i: u32) -> Result<u32, Report>;
    fn write_shared_rw_memory(&mut self, i: u32, v: u32) -> Result<(), Report>;
    fn set_input_signal(&mut self, hmsb: u32, hlsb: u32, pos: u32) -> Result<(), Report>;
    fn get_witness(&mut self, i: u32) -> Result<(), Report>;
    fn get_witness_size(&mut self) -> Result<u32, Report>;
}

impl WitnessFunctions for WitnessCalculator {
    fn init(&mut self, sanity_check: bool) -> Result<(), Report> {
        let params = [Value::I32(if sanity_check { 1 } else { 0 })];
        match self.do_call("init", &params) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn get_raw_prime(&mut self) -> Result<(), Report> {
        match self.do_call("getRawPrime", &[]) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn get_n_vars(&mut self) -> Result<u32, Report> {
        self.get_u32("getNVars", &[])
    }

    fn get_field_num_len32(&mut self) -> Result<u32, Report> {
        self.get_u32("getFieldNumLen32", &[])
    }

    fn get_version(&mut self) -> Result<u32, Report> {
        self.get_u32("getVersion", &[])
    }

    fn read_shared_rw_memory(&mut self, i: u32) -> Result<u32, Report> {
        let params = [Value::I32(i as i32)];
        self.get_u32("readSharedRWMemory", &params)
    }
    fn write_shared_rw_memory(&mut self, i: u32, v: u32) -> Result<(), Report> {
        let params = [Value::I32(i as i32), Value::I32(v as i32)];
        match self.do_call("writeSharedRWMemory", &params) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
    fn set_input_signal(&mut self, hmsb: u32, hlsb: u32, pos: u32) -> Result<(), Report> {
        let params = [
            Value::I32(hmsb as i32),
            Value::I32(hlsb as i32),
            Value::I32(pos as i32),
        ];
        match self.do_call("setInputSignal", &params) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
    fn get_witness(&mut self, i: u32) -> Result<(), Report> {
        let params = [Value::I32(i as i32)];
        match self.do_call("getWitness", &params) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
    fn get_witness_size(&mut self) -> Result<u32, Report> {
        self.get_u32("getWitnessSize", &[])
    }

    fn get_u32(&mut self, name: &str, params: &[Value]) -> Result<u32, Report> {
        match self.do_call(name, params) {
            Ok(ret) => Ok(ret[0].unwrap_i32() as u32),
            Err(e) => Err(e),
        }
    }

    fn do_call(&mut self, name: &str, params: &[Value]) -> Result<Box<[Value]>, Report> {
        let func = self.func(name);
        match func {
            Ok(f) => match f.clone().call(&mut self.store, params) {
                Ok(ret) => Ok(ret),
                Err(e) => {
                    let mut err = Report::error(e.to_string(), ReportCode::WasmRunTimeError);
                    err.add_note(format!(
                        "Was not able to call the function {} in the witness calculation wasm module",
                        name,
                    ));
                    Err(err)
                }
            },
            Err(e) => return Err(e),
        }
    }

    fn func(&mut self, name: &str) -> Result<&Function, Report> {
        match self.exports.get_function(name) {
            Ok(func) => Ok(func),
            Err(e) => {
                let mut err = Report::error(e.to_string(), ReportCode::FuncNotFound);
                err.add_note(format!(
                    "Was not able to find the function {} in the witness calculation wasm module",
                    name,
                ));
                Err(err)
            }
        }
    }
}

pub fn error(store: &mut Store) -> Function {
    #[allow(unused)]
    #[allow(clippy::many_single_char_names)]
    fn func(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) -> Result<(), RuntimeError> {
        // NOTE: We can also get more information why it is failing, see p2str etc here:
        // https://github.com/iden3/circom_runtime/blob/master/js/witness_calculator.js#L52-L64
        println!("runtime error, exiting early: {a} {b} {c} {d} {e} {f}",);
        Err(RuntimeError::user(Box::new(ExitCode(1))))
    }
    Function::new_typed(store, func)
}

// Circom 2.0
pub fn exception_handler(store: &mut Store) -> Function {
    #[allow(unused)]
    fn func(a: i32) {}
    Function::new_typed(store, func)
}

// Circom 2.0
pub fn show_memory(store: &mut Store) -> Function {
    #[allow(unused)]
    fn func() {}
    Function::new_typed(store, func)
}

// Circom 2.0
pub fn print_error_message(store: &mut Store) -> Function {
    #[allow(unused)]
    fn func() {}
    Function::new_typed(store, func)
}

// Circom 2.0
pub fn write_buffer_message(store: &mut Store) -> Function {
    #[allow(unused)]
    fn func() {}
    Function::new_typed(store, func)
}

pub fn log_signal(store: &mut Store) -> Function {
    #[allow(unused)]
    fn func(a: i32, b: i32) {}
    Function::new_typed(store, func)
}

pub fn log_component(store: &mut Store) -> Function {
    #[allow(unused)]
    fn func(a: i32) {}
    Function::new_typed(store, func)
}

fn from_array32(arr: Vec<u32>) -> BigInt {
    let mut res = BigInt::zero();
    let radix = BigInt::from(0x100000000u64);
    for &val in arr.iter() {
        res = res * &radix + BigInt::from(val);
    }
    res
}

fn to_array32(s: &BigInt, size: usize) -> Vec<u32> {
    let mut res = vec![0; size];
    let mut rem = s.clone();
    let radix = BigInt::from(0x100000000u64);
    let mut c = size;
    while !rem.is_zero() {
        c -= 1;
        res[c] = (&rem % &radix).to_u32().unwrap();
        rem /= &radix;
    }
    res
}

fn fnv(inp: &str) -> (u32, u32) {
    let mut hasher = FnvHasher::default();
    hasher.write(inp.as_bytes());
    let h = hasher.finish();

    ((h >> 32) as u32, h as u32)
}

pub fn value_to_bigint(v: serde_json::Value) -> BigInt {
    match v {
        serde_json::Value::String(inner) => {
            if inner.starts_with("0x") {
                BigInt::parse_bytes(&inner[2..].as_bytes(), 16).unwrap()
            } else {
                BigInt::parse_bytes(inner.as_bytes(), 10).unwrap()
            }
        }
        serde_json::Value::Number(inner) => BigInt::from(inner.as_u64().expect("not a u32")),
        _ => BigInt::zero(),
    }
}

/// parse_inputs accepts a JSON string and returns a HashMap of BigInts.
pub fn parse_inputs(inputs_str: &str) -> HashMap<String, Vec<BigInt>> {
    let inputs: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&inputs_str).unwrap();
    inputs
        .iter()
        .map(|(key, value)| {
            let res = match value {
                serde_json::Value::String(inner) => {
                    if inner.starts_with("0x") {
                        vec![BigInt::parse_bytes(&inner[2..].as_bytes(), 16).unwrap()]
                    } else {
                        vec![BigInt::parse_bytes(inner.as_bytes(), 10).unwrap()]
                    }
                }
                serde_json::Value::Number(inner) => {
                    vec![BigInt::from(inner.as_u64().expect("not a u32"))]
                }
                serde_json::Value::Array(inner) => {
                    inner.iter().cloned().map(value_to_bigint).collect()
                }
                _ => vec![BigInt::zero()],
            };

            (key.clone(), res)
        })
        .collect::<HashMap<_, _>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_inputs() {
        let inputs_str = r#"{"a": "1", "b": 2, "c": [3, 4], "d": "0x011"}"#;
        let inputs = parse_inputs(inputs_str);
        assert_eq!(inputs["a"], vec![BigInt::from(1)]);
        assert_eq!(inputs["b"], vec![BigInt::from(2)]);
        assert_eq!(inputs["c"], vec![BigInt::from(3), BigInt::from(4)]);
        assert_eq!(inputs["d"], vec![BigInt::from(0x11)]);
    }
}
