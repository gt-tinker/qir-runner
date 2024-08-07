// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    update_counts,
};
use std::{cell::RefCell, mem::ManuallyDrop, rc::Rc};

#[derive(Clone)]
pub struct Callable {
    func_table: *mut *mut u8,
    mem_table: *mut *mut u8,
    cap_tuple: *mut u8,
    is_adj: RefCell<bool>,
    ctls_count: RefCell<u32>,
}

#[no_mangle]
pub extern "C" fn __quantum__rt__callable_create(
    func_table: *mut *mut u8,
    mem_table: *mut *mut u8,
    cap_tuple: *mut u8,
) -> *const Callable {
    Rc::into_raw(Rc::new(Callable {
        func_table,
        mem_table,
        cap_tuple,
        is_adj: RefCell::new(false),
        ctls_count: RefCell::new(0),
    }))
}

#[no_mangle]
#[allow(clippy::cast_ptr_alignment)]
pub unsafe extern "C" fn __quantum__rt__callable_invoke(
    callable: *const Callable,
    args_tup: *mut u8,
    res_tup: *mut u8,
) {
    let call = &*callable;
    let index =
        usize::from(*call.is_adj.borrow()) + (if *call.ctls_count.borrow() > 0 { 2 } else { 0 });
    (*call
        .func_table
        .wrapping_add(index)
        .cast::<extern "C" fn(*mut u8, *mut u8, *mut u8)>())(
        call.cap_tuple,
        args_tup,
        res_tup,
    );
}

#[no_mangle]
pub unsafe extern "C" fn __quantum__rt__callable_copy(
    callable: *const Callable,
    force: bool,
) -> *const Callable {
    let rc = ManuallyDrop::new(Rc::from_raw(callable));
    if force || Rc::weak_count(&rc) > 0 {
        let copy = rc.as_ref().clone();
        Rc::into_raw(Rc::new(copy))
    } else {
        let _ = Rc::into_raw(Rc::clone(&rc));
        callable
    }
}

#[no_mangle]
pub unsafe extern "C" fn __quantum__rt__callable_make_adjoint(callable: *const Callable) {
    let call = &*callable;
    call.is_adj.replace_with(|&mut old| !old);
}

#[no_mangle]
pub unsafe extern "C" fn __quantum__rt__callable_make_controlled(callable: *const Callable) {
    let call = &*callable;
    call.ctls_count.replace_with(|&mut old| old + 1);
}

#[no_mangle]
pub unsafe extern "C" fn __quantum__rt__callable_update_reference_count(
    callable: *const Callable,
    update: i32,
) {
    update_counts(callable, update, false);
}

#[no_mangle]
pub unsafe extern "C" fn __quantum__rt__callable_update_alias_count(
    callable: *const Callable,
    update: i32,
) {
    update_counts(callable, update, true);
}

#[no_mangle]
pub unsafe extern "C" fn __quantum__rt__capture_update_reference_count(
    callable: *const Callable,
    update: i32,
) {
    let call = &*callable;
    if !call.mem_table.is_null() && !(*(call.mem_table)).is_null() {
        (*call.mem_table.cast::<extern "C" fn(*mut u8, i32)>())(call.cap_tuple, update);
    }
}

#[no_mangle]
pub unsafe extern "C" fn __quantum__rt__capture_update_alias_count(
    callable: *const Callable,
    update: i32,
) {
    let call = &*callable;
    if !call.mem_table.is_null() && !(*(call.mem_table.wrapping_add(1))).is_null() {
        (*call
            .mem_table
            .wrapping_add(1)
            .cast::<extern "C" fn(*mut u8, i32)>())(call.cap_tuple, update);
    }
}
