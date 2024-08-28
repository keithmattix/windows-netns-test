use windows::Win32::NetworkManagement::IpHelper::{SetCurrentThreadCompartmentId, GetCurrentThreadCompartmentId};
use windows::Win32::System::HostComputeNetwork::{HcnEnumerateNamespaces};
use windows::Win32::System::Com::CoTaskMemFree;
use windows_strings::{PWSTR, HSTRING, PCWSTR};
use anyhow::anyhow;
use std::fmt;

#[derive(Debug, Clone)]
struct NullBufferError;

impl fmt::Display for NullBufferError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Null buffer")
    }
}


fn main() {
    // TOOD: Remove the `.0` when the return type is fixed; WIN32_ERROR maps a u32
    // value so this should work since the memory layout is correct.
    // We assume this must not fail.
    let compartment_id : u32 = unsafe { GetCurrentThreadCompartmentId().0 };
    println!("Current compartment id: {}", compartment_id);

    // Query list of namespaces
    enumerate_namespaces().unwrap();

}

fn enumerate_namespaces() -> Result<(), anyhow::Error> {
    let mut namespace_buffer : PWSTR = PWSTR::null();
	let result_buffer : Option<*mut PWSTR> = Some(&mut PWSTR::null());

    unsafe {
        let query = r#"
        {
            "SchemaVersion": {
                "Major": 2,
                "Minor": 0
            },
            "Flags": 1,
        }
        "#;
        let h = HSTRING::from(query);
        let w = PCWSTR(h.as_ptr());
        let hr: std::result::Result<(), windows::core::Error> = HcnEnumerateNamespaces(w, &mut namespace_buffer,  result_buffer);
        let mut err_found = false;
        match hr {
            Ok(_) => {},
            Err(_) => {
                err_found = true;
            }
        };

        // check result buffer
        match result_buffer {
            Some(buffer) => {
                match convert_and_free_cotask_mem(buffer) {
                    Ok(s) => {
                        // We only return Ok() when the buffer has data, so
                        // this is an error
                        if err_found {
                            return Err(anyhow!("Error enumerating HNS namespaces. Error Code: {:?}, Details: {:?}", hr.as_ref().err().unwrap(), s));
                        } else {
                            // We didn't get a top-level error code, but the buffer still had details. Still and error
                            return Err(anyhow!("Error enumerating HNS namespaces. Error Code was unknown. Details: {:?}", s));
                        }
                    },
                    Err(e) => {
                        match e.downcast_ref::<NullBufferError>() {
                            Some(_) => {
                                // It was an empty buffer but we know we have an error from hr; return that
                                if err_found {
                                    return Err(anyhow!("Error enumerating HNS namespaces. Error Code: {:?}. Error details were", hr.as_ref().err().unwrap()));
                                }
                            },
                            None => {
                                // It was a different error, likely encountered while converting the buffer to a string
                                return Err(anyhow!("Error enumerating HNS namespaces. Error Code: {:?}. Unable to retrieve error details due to the following error: {:?}", hr.as_ref().err().unwrap(), e));
                            }
                        }

                        println!("Error converting and freeing buffer: {}", e);
                        return Err(e)
                    }
                };
            },
            None => {
                // Nothing in the result buffer so just report the error code
                if err_found {
                    println!("Result buffer is None");
                    return Err(anyhow!("Error enumerating HNS namespaces. Error Code: {:?}", hr.as_ref().err().unwrap()));
                }
            }
        };

        // Evaluate namespace buffer
        match convert_and_free_cotask_mem(&mut namespace_buffer) {
            Ok(s) => {
                // There's data in the namespace buffer
                println!("Namespace buffer: {:?}", s);
            },
            Err(e) => {
                // Definitely some issue with the buffer (either null or some other error)
                // so just return that error
                println!("Error converting and freeing namespace buffer: {}", e);
                return Err(e)
            }
        }
        Ok(())
    }
}

unsafe fn convert_and_free_cotask_mem(buffer: *mut PWSTR) -> Result<String, anyhow::Error> {
    if buffer.is_null() {
        // Safe for null pointers
        CoTaskMemFree(Some(buffer as *mut std::ffi::c_void));
        return Err(anyhow!(NullBufferError));
    }
    let x = buffer.read();

    let result : Result<String, anyhow::Error> = match PWSTR::to_string(&x) {
        Ok(s) => Ok(s),
        Err(e) => Err(anyhow!(e))
    };
    // Safe for null pointers
    CoTaskMemFree(Some(buffer as *mut std::ffi::c_void));
    result
}


