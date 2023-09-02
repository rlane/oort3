// Based on https://github.com/scrtlabs/SecretNetwork/blob/621d3899babc4741ef1ba596152c097677d246db/cosmwasm/enclaves/shared/contract-engine/src/wasm3/gas.rs
use walrus::{ir::*, FunctionBuilder, GlobalId, InitExpr, LocalFunction, ValType};

pub fn rewrite(wasm: &[u8]) -> Result<Vec<u8>, super::Error> {
    let mut module = match walrus::Module::from_buffer(wasm) {
        Ok(m) => m,
        Err(e) => {
            return Err(super::Error {
                msg: format!("{e:?}"),
            })
        }
    };

    let gas_global = module
        .globals
        .add_local(ValType::I32, true, InitExpr::Value(Value::I32(0)));

    // Rewrite each block to check and decrement gas.
    for (_, func) in module.funcs.iter_local_mut() {
        rewrite_function(func, gas_global);
    }

    // Create a reset_gas(amount) function.
    {
        let mut func = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
        let amount = module.locals.add(ValType::I32);
        func.func_body().local_get(amount).global_set(gas_global);
        let reset_gas = func.finish(vec![amount], &mut module.funcs);
        module.exports.add("reset_gas", reset_gas);
    }

    // Create a get_gas() function.
    {
        let mut func = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);
        func.func_body().global_get(gas_global);
        let get_gas = func.finish(vec![], &mut module.funcs);
        module.exports.add("get_gas", get_gas);
    }

    Ok(module.emit_wasm())
}

fn rewrite_function(func: &mut LocalFunction, gas_global: GlobalId) {
    let block_ids: Vec<_> = func.blocks().map(|(block_id, _block)| block_id).collect();
    for block_id in block_ids {
        rewrite_block(func, block_id, gas_global);
    }
}

/// Number of injected metering instructions (needed to calculate final instruction size).
const METERING_INSTRUCTION_COUNT: usize = 8;

fn rewrite_block(func: &mut LocalFunction, block_id: InstrSeqId, gas_global: GlobalId) {
    let block = func.block_mut(block_id);
    let block_instrs = &mut block.instrs;
    let block_len = block_instrs.len();
    let block_cost = block_len as i32;

    let builder = func.builder_mut();
    let mut builder = builder.dangling_instr_seq(None);
    let seq = builder
        // if unsigned(globals[gas]) < unsigned(block_cost) { throw(); }
        .global_get(gas_global)
        .i32_const(block_cost)
        .binop(BinaryOp::I32LtU)
        .if_else(
            None,
            |then| {
                then.unreachable();
            },
            |_else| {},
        )
        // globals[gas] -= block_cost;
        .global_get(gas_global)
        .i32_const(block_cost)
        .binop(BinaryOp::I32Sub)
        .global_set(gas_global);

    let mut new_instrs = Vec::with_capacity(block_len + METERING_INSTRUCTION_COUNT);
    new_instrs.append(seq.instrs_mut());

    let block = func.block_mut(block_id);
    new_instrs.extend_from_slice(block);
    block.instrs = new_instrs;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wat2wasm(wat: &str) -> Vec<u8> {
        wabt::Wat2Wasm::new()
            .convert(wat)
            .unwrap()
            .as_ref()
            .to_vec()
    }

    fn wasm2wat(wasm: &[u8]) -> String {
        String::from_utf8(
            wabt::Wasm2Wat::new()
                .convert(wasm)
                .unwrap()
                .as_ref()
                .to_vec(),
        )
        .unwrap()
        .trim()
        .to_owned()
    }

    fn check_wat(actual_wasm: &[u8], expected_wat: &str) {
        let expected_wat = expected_wat.trim();
        let actual_wat = wasm2wat(actual_wasm);
        if actual_wat != expected_wat {
            println!("Expected WAT: {expected_wat}");
            println!("Actual WAT: {actual_wat}");
        }
        assert_eq!(actual_wat, expected_wat);
    }

    #[test]
    fn test_basic() {
        let wasm = wat2wasm(
            "
(module
    (func $infinite_loop (result i32)
        (loop $loop
        br $loop)
        i32.const 1
    )
)
",
        );
        let new_wasm = rewrite(&wasm).unwrap();
        check_wat(
            &new_wasm,
            "
(module
  (type (;0;) (func (result i32)))
  (type (;1;) (func (param i32)))
  (func (;0;) (type 0) (result i32)
    global.get 0
    i32.const 2
    i32.lt_u
    if  ;; label = @1
      unreachable
    end
    global.get 0
    i32.const 2
    i32.sub
    global.set 0
    loop  ;; label = @1
      global.get 0
      i32.const 1
      i32.lt_u
      if  ;; label = @2
        unreachable
      end
      global.get 0
      i32.const 1
      i32.sub
      global.set 0
      br 0 (;@1;)
    end
    i32.const 1)
  (func (;1;) (type 1) (param i32)
    local.get 0
    global.set 0)
  (func (;2;) (type 0) (result i32)
    global.get 0)
  (global (;0;) (mut i32) (i32.const 0))
  (export \"reset_gas\" (func 1))
  (export \"get_gas\" (func 2)))
",
        );
    }
}
