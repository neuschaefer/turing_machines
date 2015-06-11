//! An LLVM-based compiler for turing machines. Coming soon.
#![feature(rustc_private)]
#![cfg(not(test))]

extern crate rustc_llvm;
extern crate libc;
extern crate turing;

use turing::{TMDesc, Movement};
use std::ffi::CString;

mod wrapper {
    use rustc_llvm as llvm;
    use std::ffi::CString;
    use libc::{c_uint, c_ulonglong};

    pub struct Module<'a> {
        raw: llvm::ModuleRef,
        #[allow(dead_code)]
        context_lifetime: &'a Context
    }

    impl<'a> Module<'a> {
        pub fn new(name: CString, context: &'a Context) -> Module<'a> {
            let raw = unsafe {
                llvm::LLVMModuleCreateWithNameInContext(
                    name.as_ptr(),
                    context.0
                )
            };
            Module {
                raw: raw,
                context_lifetime: context
            }
        }

        #[allow(unused)]
        pub fn dump(&self) {
            unsafe {
                llvm::LLVMDumpModule(self.raw)
            }
        }

        pub fn get_or_insert_function(&mut self, name: &CString, ty: Ty) -> Value {
            Value(unsafe {
                llvm::LLVMGetOrInsertFunction(self.raw, name.as_ptr(), ty.0)
            })
        }

        pub fn add_function(&mut self, name: &CString, ty: Ty) -> Value {
            Value(unsafe {
                llvm::LLVMAddFunction(self.raw, name.as_ptr(), ty.0)
            })
        }

        pub fn add_global(&mut self, ty: Ty, name: &CString) -> Value {
            Value(unsafe {
                llvm::LLVMAddGlobal(self.raw, ty.0, name.as_ptr())
            })
        }
    }

    impl<'a> Drop for Module<'a> {
        fn drop(&mut self) {
            unsafe { llvm::LLVMDisposeModule(self.raw) }
        }
    }

    pub struct Context(llvm::ContextRef);

    impl Context {
        pub fn new() -> Context {
            Context(unsafe { llvm::LLVMContextCreate() })
        }

        pub fn void_type(&self) -> Ty {
            Ty(unsafe { llvm::LLVMVoidTypeInContext(self.0) })
        }

        pub fn int8_type(&self) -> Ty {
            Ty(unsafe { llvm::LLVMInt8TypeInContext(self.0) })
        }

        pub fn int32_type(&self) -> Ty {
            Ty(unsafe { llvm::LLVMInt32TypeInContext(self.0) })
        }

        pub fn append_basic_block(&self, func: Value, name: &CString) -> BasicBlock {
            BasicBlock(unsafe {
                llvm::LLVMAppendBasicBlockInContext(self.0, func.0, name.as_ptr())
            })
        }
    }

    impl Drop for Context {
        fn drop(&mut self) {
            unsafe {
                llvm::LLVMContextDispose(self.0);
            }
        }
    }

    #[derive(Clone, Copy)]
    pub struct Value(llvm::ValueRef);

    impl Value {
        pub fn get_first_param(&self) -> Value {
            Value(unsafe {
                llvm::LLVMGetFirstParam(self.0)
            })
        }

        pub fn add_case(&self, on_val: Value, dest: BasicBlock) {
            unsafe {
                llvm::LLVMAddCase(self.0, on_val.0, dest.0)
            }
        }

        #[allow(unused)]
        pub fn dump(&self) {
            unsafe {
                llvm::LLVMDumpValue(self.0)
            }
        }

        pub fn ty(&self) -> Ty {
            Ty(unsafe {
                llvm::LLVMTypeOf(self.0)
            })
        }

        pub fn set_initializer(&self, value: Value) {
            unsafe {
                llvm::LLVMSetInitializer(self.0, value.0)
            }
        }
    }

    #[derive(Clone, Copy)]
    pub struct Ty(llvm::TypeRef);

    impl Ty {
        pub fn function_type(ret: Ty, arg: &[Ty], is_vararg: bool) -> Ty {
            Ty(unsafe {
                let ptr = arg.as_ptr() as *const llvm::TypeRef;
                let count = arg.len() as c_uint;
                llvm::LLVMFunctionType(ret.0, ptr, count, is_vararg as u32)
            })
        }

        pub fn pointer_type(&self, address_space: u32) -> Ty {
            Ty(unsafe {
                llvm::LLVMPointerType(self.0, address_space as c_uint)
            })
        }

        pub fn const_int(&self, value: u64) -> Value {
            Value(unsafe {
                let sign_extend = false as u32;
                llvm::LLVMConstInt(self.0, value as c_ulonglong, sign_extend)
            })
        }

        // This seems to be a rare case, but I'll include it for reference.
        #[allow(unused)]
        pub fn const_int_sext(&self, value: u64) -> Value {
            Value(unsafe {
                let sign_extend = true as u32;
                llvm::LLVMConstInt(self.0, value as c_ulonglong, sign_extend)
            })
        }

        pub fn const_array(&self, values: &[Value]) -> Value {
            Value(unsafe {
                let ptr = values.as_ptr() as *const llvm::ValueRef;
                let len = values.len() as c_uint;
                llvm::LLVMConstArray(self.0, ptr, len)
            })
        }
    }

    pub struct Builder<'a> {
        raw: llvm::BuilderRef,
        #[allow(dead_code)]
        context_lifetime: &'a Context
    }

    impl<'a> Builder<'a> {
        pub fn new(context: &'a Context) -> Builder<'a> {
            let raw = unsafe { llvm::LLVMCreateBuilderInContext(context.0) };
            Builder {
                raw: raw,
                context_lifetime: context
            }
        }

        pub fn position_at_end(&self, bb: BasicBlock) {
            unsafe {
                llvm::LLVMPositionBuilderAtEnd(self.raw, bb.0)
            }
        }

        pub fn build_call(&mut self, func: Value, args: &[Value], name: &CString) -> Value {
            Value(unsafe {
                let ptr = args.as_ptr() as *const llvm::ValueRef;
                let count = args.len() as c_uint;
                llvm::LLVMBuildCall(self.raw, func.0, ptr, count, name.as_ptr())
            })
        }

        pub fn build_ret(&mut self, value: Value) -> Value {
            Value(unsafe {
                llvm::LLVMBuildRet(self.raw, value.0)
            })
        }

        pub fn build_switch(&mut self, value: Value, elsebb: BasicBlock, ncases: u32) -> Value {
            Value(unsafe {
                llvm::LLVMBuildSwitch(self.raw, value.0, elsebb.0, ncases as c_uint)
            })
        }

        pub fn build_load(&mut self, ptr: Value, name: &CString) -> Value {
            Value(unsafe {
                llvm::LLVMBuildLoad(self.raw, ptr.0, name.as_ptr())
            })
        }

        pub fn build_store(&mut self, value: Value, ptr: Value) -> Value {
            Value(unsafe {
                llvm::LLVMBuildStore(self.raw, value.0, ptr.0)
            })
        }

        pub fn build_gep(&mut self, ptr: Value, indices: &[Value], name: &CString) -> Value {
            Value(unsafe {
                let ind_ptr = indices.as_ptr() as *const llvm::ValueRef;
                let ind_len = indices.len() as c_uint;
                llvm::LLVMBuildGEP(self.raw, ptr.0, ind_ptr, ind_len, name.as_ptr())
            })
        }

        pub fn build_alloca(&mut self, ty: Ty, name: CString) -> Value {
            Value(unsafe {
                llvm::LLVMBuildAlloca(self.raw, ty.0, name.as_ptr())
            })
        }

        pub fn build_br(&mut self, dest: BasicBlock) -> Value {
            Value(unsafe {
                llvm::LLVMBuildBr(self.raw, dest.0)
            })
        }

        pub fn build_global_string(&mut self, string: &CString, name: &CString) -> Value {
            Value(unsafe {
                llvm::LLVMBuildGlobalString(self.raw, string.as_ptr(), name.as_ptr())
            })
        }

        pub fn build_unreachable(&mut self) -> Value {
            Value(unsafe {
                llvm::LLVMBuildUnreachable(self.raw)
            })
        }
    }

    impl<'a> Drop for Builder<'a> {
        fn drop(&mut self) {
            unsafe { llvm::LLVMDisposeBuilder(self.raw) }
        }
    }

    #[derive(Clone, Copy)]
    struct BasicBlock(llvm::BasicBlockRef);
}


fn build_module(tmdesc: &TMDesc) {
    use wrapper::{Context, Module, Ty, Builder};

    let context = Context::new();
    let ty_void = context.void_type();
    let ty_i8 = context.int8_type();
    let ty_i32 = context.int32_type();
    let ty_i32p = ty_i32.pointer_type(0);
    let ty_i32p_i32p = Ty::function_type(ty_i32p, &[ty_i32p], false);
    let ty_i32p_i32p_p = ty_i32p_i32p.pointer_type(0);
    let ty_int = ty_i32;
    let ty_int_void = Ty::function_type(ty_int, &[], false);

    let zero_i32 = ty_i32.const_int(0u64);
    let zero_i32_twice = &[zero_i32, zero_i32];

    let mut module = Module::new(CString::new("tm").unwrap(), &context);
    let empty = &CString::new("").unwrap();

    // build the table of input symbols
    let (table, table_size) = {
        let values: Vec<_> = tmdesc.input_symbols.iter().map(
            |&sym| ty_i32.const_int(sym as u64)
        ).collect();
        let array = ty_i32.const_array(&values);
        let size_value = ty_i32.const_int(tmdesc.input_symbols.len() as u64);
        let table = module.add_global(array.ty(), &CString::new("input_symbols").unwrap());
        table.set_initializer(array);
        (table, size_value)
    };

    let tm_fail = {
        // void tm_fail(const char *str, uint32_t symbol);
        let arg_types = &[ty_i8.pointer_type(0), ty_i32];
        let ty = Ty::function_type(ty_void, arg_types, false);
        module.get_or_insert_function(&CString::new("tm_fail").unwrap(), ty)
    };

    // build the turing machine function with signature u32 *tm(u32 *TP)
    let tm_func = {
        // Map each state to a basic block like this:
        // q2:
        // switch(*TP) {
        //   case 'A': *TP = 'C'; TP++; goto q5;
        //   case 'B':            TP--; goto q4; // same symbol written back
        //   case 'C': *TP = 'A';       goto q3; // movement == None
        //   default: tm_fail("q2", *TP);
        // }
        //
        // ... except for final states, which are encoded as "return TP".

        let function = module.add_function(&CString::new("tm").unwrap(), ty_i32p_i32p);
        let mut builder = Builder::new(&context);

        let top_bb = context.append_basic_block(function, &empty);
        builder.position_at_end(top_bb);
        let tp_var = builder.build_alloca(ty_i32p, CString::new("tp").unwrap());
        builder.build_store(function.get_first_param(), tp_var);

        let state_basic_blocks: Vec<_> = tmdesc.states.iter().map(|state|
            context.append_basic_block(function, &CString::new(&state.name[..]).unwrap())
        ).collect();

        builder.build_br(state_basic_blocks[0]);

        let bb_iter = state_basic_blocks.iter();
        let state_iter = tmdesc.states.iter();
        for (&bb, state) in bb_iter.zip(state_iter) {
            builder.position_at_end(bb);

            let tp = builder.build_load(tp_var, &empty);

            if state.is_final() {
                builder.build_ret(tp);
                continue;
            }


            let default = context.append_basic_block(function, &empty);

            let current_sym = builder.build_load(tp, &empty);
            let n = state.transitions.iter().filter(|&t| t.is_some()).count();
            let switch = builder.build_switch(current_sym, default, n as u32);

            for (t, &s) in state.transitions.iter().filter_map(|t| t.as_ref())
                    .zip(tmdesc.input_symbols.iter()) {
                let sym = ty_i32.const_int(s as u64);
                let tbb = context.append_basic_block(function, &empty);
                switch.add_case(sym, tbb);

                // case 'A': *TP = 'C'; TP++; goto q5;
                builder.position_at_end(tbb);
                if s != t.symbol {
                    let new = ty_i32.const_int(t.symbol as u64);
                    builder.build_store(new, tp);
                }
                match t.movement {
                    Movement::Left | Movement::Right => {
                        let delta = ty_i32.const_int(t.movement.to_delta() as u64);
                        let new = builder.build_gep(tp, &[delta], &empty);
                        builder.build_store(new, tp_var);
                    }
                    Movement::None => ()
                }
                //let successor = tmdesc.resolve_state_index(t);
                let successor = match tmdesc.states.iter().enumerate().find(
                    |&(_, s)| s.name == t.state
                ) {
                    Some((i, _)) => i,
                    None => panic!("state \"{}\" does not exist")
                };
                builder.build_br(state_basic_blocks[successor]);
            }

            // the default case: call tm_fail with the state name and the
            // current symbol.
            builder.position_at_end(default);
            let name = builder.build_global_string(&CString::new(&state.name[..]).unwrap(), &empty);
            let name_ptr = builder.build_gep(name, zero_i32_twice, &empty);
            builder.build_call(tm_fail, &[name_ptr, current_sym], &empty);
            builder.build_unreachable();

        }

        function
    };

    // build main
    {
        let tm_run_arg_types = &[
            ty_i32p_i32p_p, // the tm function
            ty_i32p,        // table of input symbols
            ty_i32          // table size
        ];
        let tm_run_ty = Ty::function_type(ty_void, tm_run_arg_types, false);
        let tm_run = module.get_or_insert_function(&CString::new("tm_run").unwrap(), tm_run_ty);

        let mut builder = Builder::new(&context);
        let fn_main = module.add_function(&CString::new("main").unwrap(), ty_int_void);
        let bb = context.append_basic_block(fn_main, &empty);
        builder.position_at_end(bb);

        let table_ptr = builder.build_gep(table, zero_i32_twice, &empty);

        let tm_run_args = &[tm_func, table_ptr, table_size];

        builder.build_call(tm_run, tm_run_args, &empty);
        builder.build_ret(ty_i32.const_int(0));
    }

    module.dump();
}

static TM: [[&'static str; 3]; 4] = [
    ["", "ぜ", "B"],
    ["q0", "q1,B,N", "q0,ぜ,R"],
    ["q1", "STOPP,B,N", "STOPP,ぜ,L"],
    ["STOPP", "q1,B,N", "q0,ぜ,R"]
];

fn main() {
    // TODO: this should be TMDesc::from_table<I: IntoIterator>(i: I) ->
    // Result<TMDesc, TMDescError>.
    let mut desc = TMDesc::new();

    for i in TM.iter() {
        desc.handle_line(i)
    }

    build_module(&desc);
}
