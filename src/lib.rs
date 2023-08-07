mod bigintwrapper;
use bigintwrapper::BigIntWrapper;
use pyo3::prelude::*;
use num_bigint::{BigInt, Sign};

macro_rules! timeit {
    ($xpr: expr, $n: literal) => {{
        let tick = std::time::Instant::now();
        for _ in 1..$n {
            let _bigint1 = $xpr;
        }
        let tock = tick.elapsed() / $n;
        println!("Mean elapsed time for {}: {:.2?}", stringify!($xpr), tock);
        tock
    }};
}

#[pyfunction]
fn testmyshit(a: BigIntWrapper) -> PyResult<BigIntWrapper> {
    Ok(BigIntWrapper::new(a.into_inner() + 1))
}
#[pymodule]
fn pyintedit(_py: Python, m: &PyModule) -> PyResult<()> {
    let val = _py.eval("-45123", None, None).unwrap();
    let res = bigintwrapper::ffi_based_access_zero_waste_but_smart(val).unwrap();
    dbg!(res);
    let val = _py.eval("-10**300", None, None).unwrap();
    let smart = timeit!(bigintwrapper::ffi_based_access_zero_waste_but_smart(val), 1000000);
    let lowlevel = timeit!(bigintwrapper::lowlevel_access(val), 1000000);
    let baseline = timeit!(val.extract::<BigInt>(), 1000000);

    println!(
        "Ratio is {:.2}",
        smart.as_nanos() as f64 / baseline.as_nanos() as f64
    );

    m.add_function(wrap_pyfunction!(testmyshit, m)?)?;
    Ok(())
}
