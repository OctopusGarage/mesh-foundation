import unittest
import sdk


def packages():
    return [dict(name='mesh-foundation-' + n, version='0.1.0-alpha.1',
                 publish=['crates-io'], dependencies=[])
            for n in ['identity', 'storage', 'e2ee', 'connect', 'replication', 'blob', 'mailbox']]


class ReleasePolicyTests(unittest.TestCase):
    def test_alpha_and_stable_tags(self):
        for version in ['0.1.0-alpha.1', '0.1.0-beta.2', '0.1.0-rc.1', '0.1.0']:
            ps = packages()
            for p in ps:
                p['version'] = version
            self.assertEqual(sdk.validate(ps, 'v' + version), version)

    def test_invalid_or_mismatched_tags(self):
        for tag in ['v0.1.0', 'v0.1.0-alpha.01', 'v0.1.0-alpha.2', '0.1.0-alpha.1']:
            with self.assertRaises(ValueError):
                sdk.validate(packages(), tag)

    def test_rejects_mixed_versions_wrong_registry_and_missing_packages(self):
        for field, value in [('version', '0.1.0-alpha.2'), ('publish', [])]:
            ps = packages()
            ps[0][field] = value
            with self.assertRaises(ValueError):
                sdk.validate(ps)
        with self.assertRaises(ValueError):
            sdk.validate(packages()[:-1])

    def test_internal_dependencies_require_exact_matching_versions(self):
        ps = packages()
        ps[3]['dependencies'] = [dict(name=ps[0]['name'], req='*')]
        with self.assertRaises(ValueError):
            sdk.validate(ps)
        ps[3]['dependencies'][0]['req'] = '=0.1.0-alpha.1'
        sdk.validate(ps)


if __name__ == '__main__':
    unittest.main()
