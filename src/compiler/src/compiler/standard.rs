use llvm_wrap::prelude::Context;

use super::{Compiler, Runtime};

impl Compiler {
    pub(crate) fn init_std(&mut self) {
        self.env.defs.insert("print_number".to_string(),
            self.module.function_prototype(
                Some(self.pool.intern("print_number")),
                Context::function_type(
                    self.context.void_type(),
                    &[self.context.f64_type()],
                    false),
            )
        );
    }
}

impl Runtime {
    pub(crate) fn init_std(&mut self) {
        let print_number_f = self.env.defs.get("print_number")
            .expect("Cannot find print_number function.");

        unsafe {
            self.ee.add_global_mapping(print_number_f.as_value(), print_number as usize);
        }
    }
}

#[no_mangle]
extern fn print_number(value: f64) {
    println!("{}", value);
}
