use cairo_vm::vm::vm_core::VirtualMachine;

pub struct NodeEdge<'vm> {
    vm: &'vm mut VirtualMachine,
}
