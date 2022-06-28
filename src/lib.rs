use numpy::{PyArray1, IntoPyArray};
use numpy::ndarray::{ArrayViewMut1, Zip, Array1};
use pyo3::{pyclass, pymethods, pymodule, PyObject, Python, PyResult};
use pyo3::prelude::PyModule;


/// to make the memory leak happen (in python code):
/// make a Looper
/// define a pyhton function that takes an input array and returns an equal size output 
/// add the python function to the looper using add_func
/// call looper.leak_memory
/// 
/// this calls the pyhton function from rust in a loop which leaks memory for some reason?

// define a trait used to store user define callables
pub trait Func {
    fn call_func(&mut self, input: &mut ArrayViewMut1<f64>, output: &mut ArrayViewMut1<f64>);
} 

pub type BoxedFunc = Box<dyn Func>;

#[pyclass(unsendable)]
struct Looper{
    funcs: Vec<BoxedFunc>,
}

#[pymethods]
impl Looper{
    #[new]
    pub fn new() -> Looper{
        let funcs: Vec<BoxedFunc> = vec![];
        Looper{
            funcs,
        }
    }

    pub fn add_func(&mut self, callable: PyObject) {
        self.funcs.push(Box::new(UserDefinedFunc{callable}));
    }

    pub fn leek_memory(&mut self, size:usize, number:usize){
        let mut input = Array1::<f64>::zeros(size);
        let mut output = Array1::<f64>::zeros(size);
        for _i in 0..number {
            for func in &mut self.funcs {
                func.call_func(&mut input.view_mut(), &mut output.view_mut());
            }
            input.assign(&output);
        }
    }
}

struct UserDefinedFunc{
    callable: PyObject,
}

impl Func for UserDefinedFunc{
    fn call_func(&mut self, input: &mut ArrayViewMut1<f64>, output: &mut ArrayViewMut1<f64>){
        Python::with_gil(|py| -> PyResult::<()> {
            let py_ob = self.callable.call1(py, (input.to_owned().into_pyarray(py),))?;
            let py_array = py_ob.as_ref(py).downcast::<PyArray1<f64>>()?;
            let rust_array = unsafe { py_array.as_array() };

            Zip::from(output)
                .and(rust_array)
                .for_each(|o, &i| {*o += i});

            Ok(())
        });
    }
}

#[pymodule]
fn leak(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // immutable example
    m.add_class::<Looper>()?;
    
    Ok(())
}