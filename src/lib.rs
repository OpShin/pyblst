use pyo3::{prelude::*, exceptions::PyValueError};
use pyo3::types::PyBytes;

const BLST_P1_COMPRESSED_SIZE: usize = 48;

const BLST_P2_COMPRESSED_SIZE: usize = 96;

#[derive(Debug, Clone, PartialEq, thiserror::Error, miette::Diagnostic)]
pub enum Error {
    #[error("blst error {0:?}")]
    Blst(blst::BLST_ERROR),
    #[error("blst::hashToGroup")]
    HashToCurveDstTooBig,
}


impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        PyValueError::new_err(format!("blst error {:?}", err))
    }
}


#[pyclass]
pub struct BlstP1Element {
    _val: blst::blst_p1,
}

impl Clone for BlstP1Element {
    fn clone(&self) -> Self {
        BlstP1Element {
            _val: self._val.clone(),
        }
    }
}


#[pyfunction]
pub fn blst_p1_add_or_double(
    arg1: BlstP1Element,
    arg2: BlstP1Element,
) -> PyResult<BlstP1Element> {
    let mut out = blst::blst_p1::default();

    unsafe {
        blst::blst_p1_add_or_double(
            &mut out as *mut _,
            &arg1._val as *const _,
            &arg2._val as *const _,
        );
    }
    return Ok(BlstP1Element { _val: out });
}

// Compressable trait and implementations taken over with thanks from aiken
// https://github.com/aiken-lang/aiken/blob/e1d46fa8f063445da8c0372e3c031c8a11ad0b14/crates/uplc/src/machine/runtime.rs#L1769C1-L1855C2
pub trait Compressable {
    fn compress(&self) -> Vec<u8>;

    fn uncompress(bytes: &[u8]) -> Result<Self, Error>
    where
        Self: std::marker::Sized;
}

impl Compressable for blst::blst_p1 {
    fn compress(&self) -> Vec<u8> {
        let mut out = [0; BLST_P1_COMPRESSED_SIZE];

        unsafe {
            blst::blst_p1_compress(&mut out as *mut _, self);
        };

        out.to_vec()
    }

    fn uncompress(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != BLST_P1_COMPRESSED_SIZE {
            return Err(Error::Blst(blst::BLST_ERROR::BLST_BAD_ENCODING));
        }

        let mut affine = blst::blst_p1_affine::default();

        let mut out = blst::blst_p1::default();

        unsafe {
            let err = blst::blst_p1_uncompress(&mut affine as *mut _, bytes.as_ptr());

            if err != blst::BLST_ERROR::BLST_SUCCESS {
                return Err(Error::Blst(err));
            }

            blst::blst_p1_from_affine(&mut out as *mut _, &affine);

            let in_group = blst::blst_p1_in_g1(&out);

            if !in_group {
                return Err(Error::Blst(blst::BLST_ERROR::BLST_POINT_NOT_IN_GROUP));
            }
        };

        Ok(out)
    }
}

impl Compressable for blst::blst_p2 {
    fn compress(&self) -> Vec<u8> {
        let mut out = [0; BLST_P2_COMPRESSED_SIZE];

        unsafe {
            blst::blst_p2_compress(&mut out as *mut _, self);
        };

        out.to_vec()
    }

    fn uncompress(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != BLST_P2_COMPRESSED_SIZE {
            return Err(Error::Blst(blst::BLST_ERROR::BLST_BAD_ENCODING));
        }

        let mut affine = blst::blst_p2_affine::default();

        let mut out = blst::blst_p2::default();

        unsafe {
            let err = blst::blst_p2_uncompress(&mut affine as *mut _, bytes.as_ptr());

            if err != blst::BLST_ERROR::BLST_SUCCESS {
                return Err(Error::Blst(err));
            }

            blst::blst_p2_from_affine(&mut out as *mut _, &affine);

            let in_group = blst::blst_p2_in_g2(&out);

            if !in_group {
                return Err(Error::Blst(blst::BLST_ERROR::BLST_POINT_NOT_IN_GROUP));
            }
        };

        Ok(out)
    }
}

#[pyfunction]
pub fn blst_p1_uncompress(
    arg1: Bound<'_, PyBytes>,
) -> PyResult<BlstP1Element> {
    let out = blst::blst_p1::uncompress(&arg1.as_bytes())?;

    return Ok(BlstP1Element { _val: out });
}



/// A Python module implemented in Rust.
#[pymodule]
fn pyblst(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(blst_p1_add_or_double, m)?)?;
    m.add_function(wrap_pyfunction!(blst_p1_uncompress, m)?)?;
    Ok(())
}
