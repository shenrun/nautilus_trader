# -------------------------------------------------------------------------------------------------
# <copyright file="test_core_types.py" company="Nautech Systems Pty Ltd">
#  Copyright (C) 2015-2019 Nautech Systems Pty Ltd. All rights reserved.
#  The use of this source code is governed by the license as found in the LICENSE.md file.
#  https://nautechsystems.io
# </copyright>
# -------------------------------------------------------------------------------------------------

import unittest
import uuid

from nautilus_trader.core.types import ValidString, GUID


class ValidStringTests(unittest.TestCase):

    def test_equality(self):
        # Arrange
        string1 = ValidString('abc123')
        string2 = ValidString('abc123')
        string3 = ValidString('def456')

        # Act
        # Assert
        self.assertTrue('abc123', string1.value)
        self.assertTrue(string1 == string1)
        self.assertTrue(string1 == string2)
        self.assertTrue(string1 != string3)

    def test_comparison(self):
        # Arrange
        string1 = ValidString('123')
        string2 = ValidString('456')
        string3 = ValidString('abc')
        string4 = ValidString('def')

        # Act
        # Assert
        self.assertTrue(string1 <= string1)
        self.assertTrue(string1 <= string2)
        self.assertTrue(string1 < string2)
        self.assertTrue(string2 > string1)
        self.assertTrue(string2 >= string1)
        self.assertTrue(string2 >= string2)
        self.assertTrue(string3 <= string4)

    def test_hash_returns_int_type(self):
        # Arrange
        value = ValidString("abc")

        # Act
        # Assert
        self.assertEqual(int, type(hash(value)))

    def test_to_string_returns_expected_string(self):
        # Arrange
        value = ValidString("abc")

        # Act
        # Assert
        self.assertEqual("abc", value.to_string())

    def test_str_returns_expected_string(self):
        # Arrange
        value = ValidString("abc")

        # Act
        # Assert
        self.assertEqual("abc", str(value))

    def test_repr_returns_expected_string(self):
        # Arrange
        value = ValidString("abc")

        # Act
        result = repr(value)

        # Assert
        self.assertTrue(result.startswith("<ValidString(abc) object at "))
        self.assertTrue(result.endswith(">"))


class GUIDTests(unittest.TestCase):

    def test_GUIDS_passed_different_UUID_are_not_equal(self):
        # Arrange
        # Act
        guid1 = GUID(uuid.uuid4()),
        guid2 = GUID(uuid.uuid4()),

        # Assert
        self.assertNotEqual(guid1, guid2)

    def test_GUID_passed_UUID_are_equal(self):
        # Arrange
        value = uuid.uuid4()

        # Act
        guid1 = GUID(value)
        guid2 = GUID(value)

        # Assert
        self.assertEqual(guid1, guid2)
