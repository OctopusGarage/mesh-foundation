"""Keep the SDK's production surface independent from any application schema."""
import os
from pathlib import Path
import re
import unittest

ROOT = Path(os.environ.get('MESH_BOUNDARY_ROOT', Path(__file__).resolve().parents[1]))


class ArchitectureTests(unittest.TestCase):
    def test_no_application_protocol_in_production_sources(self):
        forbidden = re.compile(r'mesh-talk|mesh_talk|Mesh Talk|\bEventKind\b|\bMembershipChange\b|\bChannelMetadata\b|\blegacy_(?:event|wire|round)\b|\bpost_office\b')
        offenders = []
        for path in (ROOT / 'crates').glob('*/src/**/*.rs'):
            if forbidden.search(path.read_text()):
                offenders.append(str(path.relative_to(ROOT)))
        self.assertEqual(offenders, [], 'application protocols belong to consumer repositories')

    def test_no_application_dependencies_or_legacy_feature(self):
        for path in (ROOT / 'crates').glob('*/Cargo.toml'):
            source = path.read_text()
            self.assertNotIn('mesh-talk', source, path)
            self.assertNotIn('legacy-wire', source, path)
            self.assertNotIn('tauri', source, path)
        replication = (ROOT / 'crates/mesh-replication/Cargo.toml').read_text()
        self.assertIn('default = []', replication)

    def test_mailbox_has_no_production_identity_dependency(self):
        source = (ROOT / 'crates/mesh-mailbox/Cargo.toml').read_text().split('[dev-dependencies]')[0]
        self.assertNotIn('mesh-identity', source)
        self.assertNotIn('legacy', source)


if __name__ == '__main__':
    unittest.main()
