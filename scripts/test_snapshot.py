import json
from pathlib import Path
import subprocess
import tempfile
import unittest
import sdk_snapshot as snapshot

class SnapshotTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.source = self.root / 'sdk'
        self.source.mkdir()
        self.git('init', '-q')
        self.git('config', 'user.email', 'test@example.invalid')
        self.git('config', 'user.name', 'test')
        (self.source / 'Cargo.toml').write_text('[workspace]\n')
        (self.source / 'README.md').write_text('SDK\n')
        (self.source / 'crates/a/src').mkdir(parents=True)
        (self.source / 'crates/a/src/lib.rs').write_text('pub fn value() {}\n')
        self.commit()
        self.dest = self.root / 'vendor'

    def git(self, *args):
        return subprocess.check_output(['git', '-C', str(self.source), *args]).decode().strip()

    def commit(self):
        self.git('add', '-A'); self.git('commit', '-qm', 'fixture')

    def test_export_provenance_and_verified_update(self):
        snapshot.export_snapshot(self.source, self.dest)
        snapshot.verify_snapshot(self.dest, self.source)
        old = (self.dest / 'SOURCE').read_text()
        (self.source / 'README.md').write_text('updated SDK\n'); self.commit()
        snapshot.export_snapshot(self.source, self.dest)
        self.assertNotEqual((self.dest / 'SOURCE').read_text(), old)
        snapshot.verify_snapshot(self.dest, self.source)

    def test_modified_missing_extra_and_source_marker_are_rejected(self):
        for change in ('modified', 'missing', 'extra', 'marker'):
            dest = self.root / change
            snapshot.export_snapshot(self.source, dest)
            if change == 'modified': (dest / 'README.md').write_text('tampered')
            elif change == 'missing': (dest / 'README.md').unlink()
            elif change == 'extra': (dest / 'extra').write_text('user work')
            else: (dest / 'SOURCE').write_text('0' * 40 + '\n')
            with self.assertRaises(ValueError): snapshot.verify_snapshot(dest)
            with self.assertRaises(ValueError): snapshot.export_snapshot(self.source, dest)

    def test_manifest_cannot_claim_different_git_content(self):
        snapshot.export_snapshot(self.source, self.dest)
        path = self.dest / 'SNAPSHOT.json'
        manifest = json.loads(path.read_text())
        (self.dest / 'README.md').write_text('forged')
        import hashlib
        manifest['files']['README.md'] = hashlib.sha256(b'forged').hexdigest()
        path.write_text(json.dumps(manifest))
        snapshot.verify_snapshot(self.dest)
        with self.assertRaises(ValueError): snapshot.verify_snapshot(self.dest, self.source)

    def test_unrecognized_destination_dirty_source_and_unsafe_files_are_rejected(self):
        self.dest.mkdir(); (self.dest / 'owned').write_text('preserve')
        with self.assertRaises(ValueError): snapshot.export_snapshot(self.source, self.dest)
        self.assertEqual((self.dest / 'owned').read_text(), 'preserve')
        (self.source / 'README.md').write_text('dirty')
        with self.assertRaises(ValueError): snapshot.export_snapshot(self.source, self.root / 'dirty')
        self.commit(); (self.source / 'crates/link').symlink_to('/tmp'); self.commit()
        with self.assertRaises(ValueError): snapshot.export_snapshot(self.source, self.root / 'link')

    def test_legacy_snapshot_is_adopted_only_after_exact_source_verification(self):
        snapshot.export_snapshot(self.source, self.dest)
        (self.dest / 'SNAPSHOT.json').unlink()
        snapshot.export_snapshot(self.source, self.dest)
        snapshot.verify_snapshot(self.dest, self.source)

    def test_malformed_and_escaping_manifest_paths_are_rejected(self):
        snapshot.export_snapshot(self.source, self.dest)
        path = self.dest / 'SNAPSHOT.json'
        original = json.loads(path.read_text())
        for value in ([], dict(original, files={'../escape': '0'*64})):
            path.write_text(json.dumps(value))
            with self.assertRaises(ValueError): snapshot.verify_snapshot(self.dest)

    def test_sibling_destination_with_parent_segments_is_supported(self):
        destination = self.source.resolve() / '..' / 'sibling'
        snapshot.export_snapshot(self.source, destination)
        snapshot.verify_snapshot(destination, self.source)

    def test_consumer_checkout_preserves_snapshot_bytes_with_autocrlf(self):
        (self.source / '.gitattributes').write_text('* text=auto eol=lf\n')
        self.commit()
        consumer = self.root / 'consumer'
        consumer.mkdir()
        self.dest = consumer / 'vendor' / 'sdk'
        snapshot.export_snapshot(self.source, self.dest)
        def git(*args):
            return subprocess.check_output(['git', '-C', str(consumer), *args])
        git('init', '-q');git('config', 'user.email', 'test@example.invalid');git('config', 'user.name', 'test')
        git('add', '-A');git('commit', '-qm', 'snapshot')
        checkout = self.root / 'checkout'
        subprocess.check_call(['git', '-c', 'core.autocrlf=true', 'clone', '-q', str(consumer), str(checkout)])
        snapshot.verify_snapshot(checkout / 'vendor/sdk', self.source)
