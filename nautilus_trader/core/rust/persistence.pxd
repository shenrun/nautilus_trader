# Warning, this file is autogenerated by cbindgen. Don't modify this manually. */

from cpython.object cimport PyObject
from libc.stdint cimport uintptr_t
from nautilus_trader.core.rust.model cimport QuoteTick_t, Bar_t

cdef extern from "../includes/persistence.h":

    cdef struct Vec_QuoteTick:
        QuoteTick_t *ptr;
        uintptr_t len;
        uintptr_t cap;

    cdef struct Vec_Bar:
        Bar_t *ptr;
        uintptr_t len;
        uintptr_t cap;

    const QuoteTick_t *index_quote_tick_vector(const Vec_QuoteTick *ptr, uintptr_t i);

    Vec_QuoteTick read_parquet_ticks(PyObject *path, PyObject *filter_exprs);

    const Bar_t *index_bar_vector(const Vec_Bar *ptr, uintptr_t i);

    Vec_Bar read_parquet_bars(PyObject *path, PyObject *filter_exprs);
