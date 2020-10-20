# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2020 Nautech Systems Pty Ltd. All rights reserved.
#  https://nautechsystems.io
#
#  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
#  You may not use this file except in compliance with the License.
#  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
# -------------------------------------------------------------------------------------------------

from nautilus_trader.core.decimal cimport Decimal
from nautilus_trader.model.currency cimport Currency


cdef class Quantity(Decimal):

    @staticmethod
    cdef inline Quantity from_float_c(double value, int precision)
    cpdef double as_double(self) except *
    cpdef str to_string(self)
    cpdef str to_string_formatted(self)


cdef class Price(Decimal):

    @staticmethod
    cdef inline Price from_float_c(double value, int precision)
    cpdef double as_double(self) except *
    cpdef str to_string(self)


cdef class Money(Decimal):
    cdef readonly Currency currency

    cpdef double as_double(self) except *
    cpdef str to_string(self)
    cpdef str to_string_formatted(self)
