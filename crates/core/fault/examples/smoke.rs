//! Minimal entry: turns one supervised panic into a host fault and prints it.

use tg_core_fault::{HostFaultDomain, HostFaultPhase, catch_host_fault};

fn main() {
  std::panic::set_hook(Box::new(|_| {}));
  let fault = catch_host_fault(HostFaultPhase::Runtime, HostFaultDomain::Other, || -> () {
    panic!("smoke panic")
  })
  .expect_err("panic must be converted into a fault");
  assert!(fault.to_string().contains("smoke panic"));
  let value = catch_host_fault(HostFaultPhase::Runtime, HostFaultDomain::Other, || 7).unwrap();
  assert_eq!(value, 7);
  println!("fault ok: {fault}");
}
