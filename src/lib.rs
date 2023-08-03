mod bigintwrapper;
use bigintwrapper::BigIntWrapper;
use pyo3::prelude::*;

#[pyfunction]
fn testmyshit(a: BigIntWrapper) -> PyResult<BigIntWrapper> {
    Ok(BigIntWrapper::new(a.into_inner() * 10))
}
#[pymodule]
fn pyintedit(_py: Python, m: &PyModule) -> PyResult<()> {
    let val = _py.eval("-65535**150", None, None).unwrap();

    let before = std::time::Instant::now();
    let itrs = 100000;
    for _ in 1..itrs {
        let _bigint1 = bigintwrapper::ffi_based_access(val);
    }
    let time1 = before.elapsed() / itrs;
    println!("Mean elapsed time for ffi_based_access: {:.2?}", time1);

    let before = std::time::Instant::now();
    for _ in 1..itrs {
        let _bigint2 = bigintwrapper::lowlevel_access(val);
    }
    let time2 = before.elapsed() / itrs;
    println!("Mean elapsed time for lowlevel_access: {:.2?}", time2);
    println!(
        "Ratio is {:2}",
        time1.as_nanos() as f64 / time2.as_nanos() as f64
    );

    m.add_function(wrap_pyfunction!(testmyshit, m)?)?;
    Ok(())
}
