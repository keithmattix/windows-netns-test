use windows::Win32::NetworkManagement::IpHelper::{SetCurrentThreadCompartmentId, GetCurrentThreadCompartmentId};
use hcn::{api, get_namespace, schema::*};
use std::fmt;


fn main() {
    // TOOD: Remove the `.0` when the return type is fixed; WIN32_ERROR maps a u32
    // value so this should work since the memory layout is correct.
    // We assume this must not fail.
    let compartment_id : u32 = unsafe { GetCurrentThreadCompartmentId().0 };
    println!("Current compartment id: {}", compartment_id);

    // Query list of namespaces
    let query = HostComputeQuery::default();
    let query = serde_json::to_string(&query).unwrap();
    let namespaces = api::enumerate_namespaces(&query).unwrap();
    let namespaces : Vec<String> = serde_json::from_str(&namespaces).unwrap();
    let ns = get_namespace(namespaces.first().unwrap().as_str()).unwrap();
    println!("First Namespace details: {:?}", ns);
    // Namespaced ID == Compartment ID so we can use that to change our compartment
    unsafe {
        let error = SetCurrentThreadCompartmentId(ns.namespace_id.unwrap());
        if error.0 != 0 {
            panic!("Error setting compartment id: {}", error.0);
        }
        println!("Printing from inside Compartment ID {}", GetCurrentThreadCompartmentId().0);
        let error = SetCurrentThreadCompartmentId(compartment_id);
        if error.0 != 0 {
            panic!("Error setting compartment id: {}", error.0);
        }

        println!("Back in original compartment ID {}", GetCurrentThreadCompartmentId().0);
    }


}


