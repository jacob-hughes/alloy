// run-pass
// ignore-tidy-linelength
#![feature(gc)]

use std::mem;
use std::gc::NoFinalize;

struct HasDrop;

impl Drop for HasDrop {
    fn drop(&mut self) {}
}

struct HasDropNoFinalize;

impl Drop for HasDropNoFinalize {
    fn drop(&mut self) {}
}

impl NoFinalize for HasDropNoFinalize {}

struct FinalizedContainer<T>(T);

impl<T> Drop for FinalizedContainer<T> {
    fn drop(&mut self) {}
}

const CONST_U8: bool = mem::needs_finalizer::<u8>();
const CONST_STRING: bool = mem::needs_finalizer::<String>();
const CONST_FINALIZABLE: bool = mem::needs_finalizer::<HasDrop>();
const CONST_UNFINALIZABLE: bool = mem::needs_finalizer::<HasDropNoFinalize>();

static STATIC_U8: bool = mem::needs_finalizer::<u8>();
static STATIC_STRING: bool = mem::needs_finalizer::<String>();
static STATIC_FINALIZABLE: bool = mem::needs_finalizer::<HasDrop>();
static STATIC_UNFINALIZABLE: bool = mem::needs_finalizer::<HasDropNoFinalize>();

static BOX_TRIVIAL: bool = mem::needs_finalizer::<Box<usize>>();
static BOX_FINALIZABLE: bool = mem::needs_finalizer::<Box<HasDrop>>();
static BOX_UNFINALIZABLE: bool = mem::needs_finalizer::<Box<HasDropNoFinalize>>();
static BOX_TUPLE_FINALIZABLE: bool = mem::needs_finalizer::<Box<(HasDrop, HasDrop)>>();
static BOX_TUPLE_UNFINALIZABLE: bool = mem::needs_finalizer::<Box<(HasDropNoFinalize, HasDropNoFinalize)>>();

static VEC_TRIVIAL: bool = mem::needs_finalizer::<Vec<usize>>();
static VEC_FINALIZABLE: bool = mem::needs_finalizer::<Vec<HasDrop>>();
static VEC_UNFINALIZABLE: bool = mem::needs_finalizer::<Vec<HasDropNoFinalize>>();
static VEC_TUPLE_UNFINALIZABLE: bool = mem::needs_finalizer::<Vec<(HasDropNoFinalize, usize)>>();
static VEC_TUPLE_FINALIZABLE: bool = mem::needs_finalizer::<Vec<(HasDrop, HasDrop)>>();
static VEC_TUPLE_CONTAINS_FINALIZABLE: bool = mem::needs_finalizer::<Vec<(HasDrop, usize)>>();

static VEC_VEC_FINALIZABLE: bool = mem::needs_finalizer::<Vec<Vec<HasDrop>>>();
static VEC_VEC_UNFINALIZABLE: bool = mem::needs_finalizer::<Vec<Vec<HasDropNoFinalize>>>();
static VEC_STRING: bool = mem::needs_finalizer::<Vec<String>>();
static VEC_BOX_FINALIZABLE: bool = mem::needs_finalizer::<Vec<Box<HasDrop>>>();
static VEC_BOX_UNFINALIZABLE: bool = mem::needs_finalizer::<Vec<Box<HasDropNoFinalize>>>();


static OUTER_NEEDS_FINALIZING: bool = mem::needs_finalizer::<FinalizedContainer<Vec<HasDropNoFinalize>>>();

fn main() {
    assert!(!CONST_U8);
    assert!(!CONST_STRING);
    assert!(CONST_FINALIZABLE);
    assert!(!CONST_UNFINALIZABLE);

    assert!(!STATIC_U8);
    assert!(!STATIC_STRING);
    assert!(STATIC_FINALIZABLE);
    assert!(!STATIC_UNFINALIZABLE);

    assert!(!BOX_TRIVIAL);
    assert!(BOX_FINALIZABLE);
    assert!(!BOX_UNFINALIZABLE);
    assert!(BOX_TUPLE_FINALIZABLE);
    assert!(!BOX_TUPLE_UNFINALIZABLE);

    assert!(!VEC_TRIVIAL);
    assert!(VEC_FINALIZABLE);
    assert!(!VEC_UNFINALIZABLE);
    assert!(!VEC_TUPLE_UNFINALIZABLE);
    assert!(VEC_TUPLE_FINALIZABLE);
    assert!(VEC_TUPLE_CONTAINS_FINALIZABLE);

    assert!(VEC_VEC_FINALIZABLE);
    assert!(!VEC_VEC_UNFINALIZABLE);
    assert!(!VEC_STRING);
    assert!(VEC_BOX_FINALIZABLE);
    assert!(!VEC_BOX_UNFINALIZABLE);

    assert!(OUTER_NEEDS_FINALIZING);
}
