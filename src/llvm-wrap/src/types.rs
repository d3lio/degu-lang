use super::llvm_ref::LlvmRef;

#[derive(Clone, Copy)]
pub struct Type {
    pub(crate) ptr: <Self as LlvmRef>::Ref,
}
