# Copyright 2012 Martin Pool
# Licensed under the Apache License, Version 2.0 (the "License").

"""Unit test Archive marker/metadata"""


from __future__ import absolute_import

import errno
import os.path
import unittest

from duralib.archive import (
    Archive,
    BadArchiveHeader,
    NoSuchArchive,
    )
from duralib.tests.base import DuraTestCase
from duralib.tests.durafixtures import (
    EmptyArchive,
    PopulatedArchive,
    )


class TestArchive(DuraTestCase):

    def setUp(self):
        super(TestArchive, self).setUp()
        self.archive_path = self.subpath("testarchive")

    def test_create_archive(self):
        new_archive = Archive.create(self.archive_path)
        self.assertEquals(self.archive_path, new_archive.path)
        self.assertTrue(os.path.isdir(self.archive_path))
        self.assertTrue(
            os.path.isfile(
                os.path.join(self.archive_path, "DURA-ARCHIVE")))

    def test_archive_repr(self):
        archive = self.useFixture(EmptyArchive()).archive
        self.assertRegexpMatches(
            repr(archive),
            r"Archive\('.*'\)")

    def test_corrupt_magic(self):
        with file(os.path.join(self.tmpdir, 'DURA-ARCHIVE'), 'w') as f:
            f.write('some garbage')
        with self.assertRaises(BadArchiveHeader) as e:
            Archive.open(self.tmpdir)
        self.assertRegexpMatches(
            str(e.exception),
            "Bad archive header: " + self.tmpdir)

    def test_reopen_archive(self):
        Archive.create(self.archive_path)
        second = Archive.open(self.archive_path)
        self.assertEquals(self.archive_path, second.path)

    def test_open_nonexistent(self):
        # Don't create it
        with self.assertRaises(NoSuchArchive) as catcher:
            Archive.open(self.archive_path)
        self.assertRegexpMatches(str(catcher.exception),
            r"No such archive: .*testarchive.*%s"
            % os.strerror(errno.ENOENT))

    def test_open_bad_magic(self):
        orig_archive = Archive.create(self.archive_path)
        with file(orig_archive._header_path, "wb") as f:
            f.write("not this!")
        with self.assertRaises(BadArchiveHeader) as ar:
            Archive.open(self.archive_path)
        self.assertEquals(
            "Bad archive header: %s" % orig_archive.relpath('DURA-ARCHIVE'),
            str(ar.exception))

    def test_list_bands_empty(self):
        archive = self.useFixture(EmptyArchive()).archive
        self.assertEquals([], archive.list_bands())

    def test_list_bands_populated(self):
        archive = self.useFixture(PopulatedArchive()).archive
        self.assertEquals(
            ["0000", "0001", "0002"],
            archive.list_bands())

    def test_create_band(self):
        archive = self.useFixture(EmptyArchive()).archive
        band = archive.create_band()
        self.assertEquals("0000", band.band_number)
        self.assertEquals(archive, band.archive)
        self.assertEquals(
            os.path.join(archive.path, 'b0000'),
            band.path)
        self.assertTrue(
            os.path.isdir(band.path))
        self.assertEquals(
            ["0000"], archive.list_bands())
        self.assertEquals(
            ["BAND-HEAD"], os.listdir(band.path))

    def test_create_band_repeated(self):
        archive = self.useFixture(EmptyArchive()).archive
        num_bands = 17
        unused_bands = [archive.create_band() for i in range(num_bands)]
        self.assertEquals(
            ["%04d" % i for i in range(num_bands)],
            archive.list_bands())


if __name__ == '__main__':
    unittest.main()