#!/usr/bin/env python3
# -------------------------------------------------------------------------------------------------
# <copyright file="responses.pxd" company="Invariance Pte">
#  Copyright (C) 2018-2019 Invariance Pte. All rights reserved.
#  The use of this source code is governed by the license as found in the LICENSE.md file.
#  http://www.invariance.com
# </copyright>
# -------------------------------------------------------------------------------------------------

# cython: language_level=3, boundscheck=False, wraparound=False, nonecheck=False

from inv_trader.core.message cimport Response
from inv_trader.model.objects cimport Symbol, BarSpecification


cdef class TickDataResponse(Response):
    """
    Represents a response of historical tick data.
    """
    cdef readonly Symbol symbol
    cdef readonly bytearray ticks


cdef class BarDataResponse(Response):
    """
    Represents a response of historical bar data.
    """
    cdef readonly Symbol symbol
    cdef readonly BarSpecification bar_spec
    cdef readonly list bars


cdef class InstrumentResponse(Response):
    """
    Represents a response of instrument data.
    """
    cdef readonly list instruments
