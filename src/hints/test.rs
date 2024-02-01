#[cfg(test)]
mod test {
    use cairo_vm::{
        serde::deserialize_program::ApTracking,
        types::exec_scope::ExecutionScopes,
    };
    use crate::hints::*;

    macro_rules! segments {
        ($( (($si:expr, $off:expr), $val:tt) ),* $(,)? ) => {
            {
                let memory = memory!($( (($si, $off), $val) ),*);
                $crate::vm::vm_memory::memory_segments::MemorySegmentManager {
                    memory,
                    segment_sizes: HashMap::new(),
                    segment_used_sizes: None,
                    public_memory_offsets: HashMap::new(),
                }

            }

        };
    }

    macro_rules! references {
        ($num: expr) => {{
            let mut references = cairo_vm::stdlib::collections::HashMap::<usize, HintReference>::new();
            for i in 0..$num {
                references.insert(i as usize, HintReference::new_simple((i as i32 - $num)));
            }
            references
        }};
    }

    macro_rules! add_segments {
        ($vm:expr, $n:expr) => {
            for _ in 0..$n {
                $vm.segments.add();
            }
        };
    }

    macro_rules! ids_data {
        ( $( $name: expr ),* ) => {
            {
                let ids_names = vec![$( $name ),*];
                let references = references!(ids_names.len() as i32);
                let mut ids_data = cairo_vm::stdlib::collections::HashMap::<cairo_vm::stdlib::string::String, HintReference>::new();
                for (i, name) in ids_names.iter().enumerate() {
                    ids_data.insert(cairo_vm::stdlib::string::ToString::to_string(name), references.get(&i).unwrap().clone());
                }
                
                println!("IDS {:?}", ids_data);
                ids_data
            }
        };
    }
    #[test]
    fn test_is_n_ge_two_fail() {
        let mut vm = VirtualMachine::new(false);
        let ids_data = ids_data!["n"];
        let ap_tracking = ApTracking::default();
        let mut exec_scopes: ExecutionScopes = ExecutionScopes::new();

        vm.set_fp(5);
        //Insert ids into memory
        vm.segments = segments![((1, 4), 1)];
        add_segments!(vm, 1);
        //Create ids_data
        let ids_data = ids_data!["a"];

        let hola = insert_value_from_var_name("n", Felt252::TWO, &mut vm, &ids_data, &ap_tracking);
        println!("HOLAAAA {:?}", hola);
        is_n_ge_two(&mut vm, &mut exec_scopes, &ids_data, &ap_tracking,  &Default::default()).expect("is_n_ge_two() failed");

    }
}