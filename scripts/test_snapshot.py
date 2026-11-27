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

    def test_failed_install_and_failed_restore_preserve_recoverable_backup(self):
        from unittest.mock import patch
        snapshot.export_snapshot(self.source, self.dest)
        original = Path.rename
        def fail_install_and_restore(path, target):
            if path.name == 'snapshot' or path == snapshot.backup_path(self.dest):
                raise OSError('injected rename failure')
            return original(path, target)
        with patch.object(Path, 'rename', fail_install_and_restore):
            with self.assertRaises(OSError): snapshot.export_snapshot(self.source, self.dest)
        backup = snapshot.backup_path(self.dest)
        snapshot.verify_snapshot(backup, self.source)
        self.assertFalse(self.dest.exists())
        snapshot.export_snapshot(self.source, self.dest)
        snapshot.verify_snapshot(self.dest, self.source)
        self.assertFalse(backup.exists())

    def test_destination_has_one_writer_and_lease_is_released_after_error(self):
        with snapshot.destination_lock(self.dest):
            with self.assertRaisesRegex(ValueError, 'busy'):
                snapshot.export_snapshot(self.source, self.dest)
        snapshot.export_snapshot(self.source, self.dest)
        snapshot.verify_snapshot(self.dest, self.source)

    def test_verified_backup_after_completed_install_is_cleaned_on_retry(self):
        import shutil
        snapshot.export_snapshot(self.source, self.dest)
        backup = snapshot.backup_path(self.dest)
        shutil.copytree(self.dest, backup)
        snapshot.export_snapshot(self.source, self.dest)
        self.assertFalse(backup.exists())
        snapshot.verify_snapshot(self.dest, self.source)

    def test_process_exit_releases_destination_lease(self):
        import sys
        code = ('import sys;from pathlib import Path;'
                'sys.path.insert(0,sys.argv[1]);import sdk_snapshot;'
                '\nwith sdk_snapshot.destination_lock(Path(sys.argv[2])):\n'
                ' print("locked",flush=True)\n sys.stdin.read()\n')
        with subprocess.Popen([sys.executable, '-u', '-c', code,
                               str(Path(snapshot.__file__).parent), str(self.dest)],
                              stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                              text=True) as child:
            try:
                self.assertEqual(child.stdout.readline().strip(), 'locked')
                with self.assertRaisesRegex(ValueError, 'busy'):
                    snapshot.export_snapshot(self.source, self.dest)
            finally:
                child.terminate(); child.wait(timeout=5)
        snapshot.export_snapshot(self.source, self.dest)
        snapshot.verify_snapshot(self.dest, self.source)

    def test_lease_identity_is_independent_of_process_temporary_directory(self):
        import os
        import sys
        other=self.root/'other-temp';other.mkdir()
        code=('import sys;from pathlib import Path;sys.path.insert(0,sys.argv[1]);import sdk_snapshot;'
              '\ntry:\n with sdk_snapshot.destination_lock(Path(sys.argv[2])):sys.exit(9)\n'
              'except ValueError as error:\n sys.exit(0 if "busy" in str(error) else 8)\n')
        env=dict(os.environ,TMPDIR=str(other),TEMP=str(other),TMP=str(other))
        with snapshot.destination_lock(self.dest):
            result=subprocess.run([sys.executable,'-c',code,str(Path(snapshot.__file__).parent),str(self.dest)],env=env,timeout=10)
            self.assertEqual(result.returncode,0)

    def test_unknown_recovery_directory_is_preserved_and_rejected(self):
        snapshot.export_snapshot(self.source,self.dest)
        backup=snapshot.backup_path(self.dest);backup.mkdir()
        owned=backup/'unrelated';owned.write_text('preserve')
        with self.assertRaises(ValueError):snapshot.export_snapshot(self.source,self.dest)
        self.assertEqual(owned.read_text(),'preserve')
        snapshot.verify_snapshot(self.dest,self.source)

    def test_failed_install_restores_the_previous_source_revision(self):
        from unittest.mock import patch
        snapshot.export_snapshot(self.source,self.dest)
        old=snapshot.verify_snapshot(self.dest,self.source)
        (self.source/'README.md').write_text('new revision');self.commit()
        original=Path.rename
        def fail_install(path,target):
            if path.name=='snapshot':raise OSError('injected install failure')
            return original(path,target)
        with patch.object(Path,'rename',fail_install):
            with self.assertRaises(OSError):snapshot.export_snapshot(self.source,self.dest)
        self.assertEqual(snapshot.verify_snapshot(self.dest,self.source),old)
        self.assertFalse(snapshot.backup_path(self.dest).exists())
