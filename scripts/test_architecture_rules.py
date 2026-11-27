"""Exercise architecture rules with representative valid and invalid inputs."""
import unittest
import test_architecture as architecture


class ArchitectureRuleTests(unittest.TestCase):
    def test_compact_application_names_are_rejected(self):
        for name in ['MeshTalk', 'mesh_talk', 'mesh-talk', 'Mesh Talk']:
            with self.subTest(name=name):
                self.assertTrue(architecture.application_protocol_hits('mesh-replication/src/lib.rs', f'use {name}::Event;'))

    def test_historical_literals_are_allowed_only_at_the_reviewed_location(self):
        literal = 'b"MeshTalk-Ratchet-RK"'
        self.assertFalse(architecture.application_protocol_hits('mesh-e2ee/src/ratchet/kdf.rs', literal))
        self.assertTrue(architecture.application_protocol_hits('mesh-replication/src/lib.rs', literal))
        self.assertTrue(architecture.application_protocol_hits('mesh-e2ee/src/ratchet/kdf.rs', literal + '; MeshTalk::Event'))
        self.assertTrue(architecture.application_protocol_hits('mesh-e2ee/src/ratchet/kdf.rs', 'b"MeshTalk-NewProfile"'))

    def test_renamed_and_target_specific_application_dependencies_are_rejected(self):
        manifest = {
            'dependencies': {'innocent_alias': {'package': 'mesh-talk-protocol', 'version': '1'}},
            'target': {'cfg(unix)': {'build-dependencies': {'ui': {'package': 'tauri', 'version': '2'}}}},
        }
        self.assertEqual(set(architecture.application_dependencies(manifest)), {'mesh-talk-protocol', 'tauri'})

    def test_generic_dependencies_and_package_metadata_are_allowed(self):
        manifest = {'package': {'description': 'Extracted from Mesh Talk'}, 'dependencies': {'mesh-foundation-e2ee': '1', 'serde': {'version': '1'}}}
        self.assertEqual(architecture.application_dependencies(manifest), [])

    def test_source_checks_read_utf8_even_with_a_legacy_windows_locale(self):
        from pathlib import Path
        import tempfile
        from unittest.mock import patch
        original = Path.read_text
        def windows_read(path, *args, **kwargs):
            kwargs.setdefault('encoding', 'cp1252')
            return original(path, *args, **kwargs)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'crates/example/src/lib.rs'
            source.parent.mkdir(parents=True)
            source.write_bytes(b'// indexed value x\xe2\x82\x81\n')
            with patch.object(architecture, 'ROOT', root), patch.object(Path, 'read_text', windows_read):
                architecture.ArchitectureTests().test_no_application_protocol_in_production_sources()
