// ************************************************************************************************
// use
// ************************************************************************************************

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use super::TimeStats;

// ************************************************************************************************
// macro to get function name
// ************************************************************************************************

#[macro_export]
macro_rules! function {
    () => {{
        fn f() {}
        #[inline(always)]
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        type_name_of(f)
        // name.strip_suffix("::f").unwrap()
    }};
}

// ************************************************************************************************
// struct
// ************************************************************************************************

#[derive(Debug, Clone)]
pub struct FunctionTimer {
    function_name: &'static str,
    start_time: Instant,
    time_stats: Rc<RefCell<TimeStats>>,
}

// ************************************************************************************************
// impl
// ************************************************************************************************

impl FunctionTimer {
    pub fn start(function_name: &'static str, time_stats: Rc<RefCell<TimeStats>>) -> Self {
        Self {
            function_name,
            start_time: Instant::now(),
            time_stats,
        }
    }
}

// ************************************************************************************************
// impl Drop
// ************************************************************************************************

impl Drop for FunctionTimer {
    fn drop(&mut self) {
        self.time_stats
            .borrow_mut()
            .update_time(self.function_name, self.start_time.elapsed());
    }
}
