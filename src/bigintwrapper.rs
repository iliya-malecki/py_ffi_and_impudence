use num_bigint::{BigInt, Sign};
use pyo3::prelude::*;
use pyo3::AsPyPointer;

pub struct BigIntWrapper(BigInt);

impl BigIntWrapper {
    pub fn new(value: BigInt) -> Self {
        BigIntWrapper(value)
    }

    pub fn into_inner(self) -> BigInt {
        self.0
    }
}

#[cfg(feature = "slow")]
impl<'source> FromPyObject<'source> for BigIntWrapper {
    fn extract(obj: &'source PyAny) -> PyResult<BigIntWrapper> {
        Ok(BigIntWrapper(ffi_based_access(obj)?))
    }
}

#[cfg(feature = "fast")]
impl<'source> FromPyObject<'source> for BigIntWrapper {
    fn extract(obj: &'source PyAny) -> PyResult<BigIntWrapper> {
        Ok(BigIntWrapper(lowlevel_access(obj)?))
    }
}

impl IntoPy<PyObject> for BigIntWrapper {
    fn into_py(self, py: Python<'_>) -> PyObject {
        let bytes = self.into_inner().to_signed_bytes_le();

        unsafe {
            PyObject::from_owned_ptr(
                py,
                pyo3::ffi::_PyLong_FromByteArray(bytes.as_ptr(), bytes.len(), 1, 1),
            )
        }
    }
}

pub fn ffi_based_access(obj: &PyAny) -> PyResult<BigInt> {
    let ptr = obj.as_ptr();
    if ptr.is_null() {
        return Err(PyErr::fetch(obj.py()));
    }
    let nbytes = unsafe {
        // round up and add an extra byte just for the sign bit because
        // afaik at this moment there is no ffi function to get the sign
        (pyo3::ffi::_PyLong_NumBits(ptr) + 15) / 8
    };

    let mut buffer = Vec::<u8>::with_capacity(nbytes);
    unsafe {
        let retcode = pyo3::ffi::_PyLong_AsByteArray(
            ptr as *mut pyo3::ffi::PyLongObject,
            buffer.as_mut_ptr(),
            nbytes,
            1,
            1,
        );
        if retcode == -1 {
            return Err(PyErr::fetch(obj.py()));
        }
        buffer.set_len(nbytes);
    }

    Ok(BigInt::from_signed_bytes_le(&buffer))
}

pub fn ffi_based_access_zero_waste(obj: &PyAny) -> PyResult<BigInt> {
    let ptr = obj.as_ptr();
    if ptr.is_null() {
        return Err(PyErr::fetch(obj.py()));
    }
    let mut nbytes = unsafe {pyo3::ffi::_PyLong_NumBits(ptr)};

    if nbytes == 0 {
        return Ok(BigInt::from(0))
    }
    // round up and add up to 4 bytes (but never 0) just for the sign bit because
    // afaik at this moment there is no ffi function to get the sign
    nbytes = (nbytes + 32) / 32 * 4;

    let mut buffer = Vec::<u8>::with_capacity(nbytes);
    unsafe {
        let retcode = pyo3::ffi::_PyLong_AsByteArray(
            ptr as *mut pyo3::ffi::PyLongObject,
            buffer.as_mut_ptr(),
            nbytes,
            1,
            1,
        );
        if retcode == -1 {
            return Err(PyErr::fetch(obj.py()));
        }
        buffer.set_len(nbytes);
    }

    let sign = if buffer[nbytes-1] & 0x80 != 0 {
        buffer.iter_mut().for_each(|element| *element = !*element);
        Sign::Minus
    } else {
        Sign::Plus
    };


    assert!(buffer.len() % 4 == 0);
    assert!(buffer.capacity() % 4 == 0);

    let mut num =  BigInt::new(
        sign,
        unsafe {
            let vec = Vec::from_raw_parts(
                buffer.as_mut_ptr() as *mut u32,
                buffer.len() / 4,
                buffer.capacity() / 4
            );
            std::mem::forget(buffer);
            vec
        }
    );

    if num.sign() == Sign::Minus {
        num -= 1;
    }

    Ok(num)

}

pub fn pytonormal(pyints: &[u32]) -> Vec<u32> {
    let mut newints: Vec<u32> = Vec::with_capacity(pyints.len());
    let mut current = 0;
    for (i, &next) in pyints.iter().enumerate() {
        // maps from 0..inf to 32..0.step_by(2) but with a quirk
        let magic_num = (32 - (i * 2) % 32) % 32;
        if magic_num != 0 {
            newints.push(current | (next << magic_num));
        }
        current = next >> ((32 - magic_num) % 32);
    }
    newints.push(current);

    newints
}

pub fn lowlevel_access(obj: &PyAny) -> PyResult<BigInt> {
    let ptr = obj.as_ptr();
    if ptr.is_null() {
        return Err(PyErr::fetch(obj.py()));
    }
    let lv_tag_ptr = unsafe { (ptr as *const u8).offset(16) as *const i64 };
    let lv_tag = unsafe { *lv_tag_ptr };
    let digits_ptr = unsafe { lv_tag_ptr.offset(1) as *mut u32 };
    let digitcount = lv_tag.abs() as usize;
    let negative = lv_tag < 0;

    if digitcount == 0 {
        return Ok(BigInt::from(0));
    }
    let digits_as_full_u32 =
        unsafe { pytonormal(std::slice::from_raw_parts(digits_ptr, digitcount)) };

    if negative {
        Ok(BigInt::new(Sign::Minus, digits_as_full_u32))
    } else {
        Ok(BigInt::new(Sign::Plus, digits_as_full_u32))
    }
}
