"""Release integration tests use disposable Git repositories, never the checkout."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('release', Path(__file__).with_name('release.py'))
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class ReleaseTests(unittest.TestCase):
    def test_severity(self):
        for text, expected in [('fix: repair', 1), ('feat(cli): add', 2), ('fix!: break', 3), ('chore: tidy', 0), ('fix: x\n\nBREAKING CHANGE: x', 3)]:
            self.assertEqual(release.severity(text), expected)

    def test_lifecycle(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            def git(*args):
                return subprocess.check_output(['git', *args], cwd=root, text=True, stderr=subprocess.DEVNULL).strip()
            git('init', '-b', 'main')
            git('config', 'user.name', 'Test')
            git('config', 'user.email', 'test@example.com')
            for name, text in {'.version': '0.2.0\n', 'Cargo.toml': '[package]\nname = "sigma"\nversion = "0.2.0"\n', 'Cargo.lock': '[[package]]\nname = "sigma"\nversion = "0.2.0"\n', 'CHANGELOG.md': '# Changelog\n'}.items():
                (root / name).write_text(text)
            git('add', '.')
            git('commit', '-m', 'chore: baseline')
            git('tag', 'v0.2.0')
            (root / 'source').write_text('feature')
            git('add', '.')
            git('commit', '-m', 'feat: new feature')
            (root / 'landing/src/lib').mkdir(parents=True)
            (root / 'landing/src/lib/prompt-data.js').write_text('https://example.com/v0.2.0-unstable/sigma')
            git('add', '.')
            git('commit', '-m', 'docs: download link')
            before = git('rev-parse', 'HEAD')
            self.assertEqual(release.release(root)['version'], '0.3.0')
            self.assertEqual(git('rev-parse', 'HEAD'), before)
            result = release.release(root, write=True)
            self.assertEqual(result['tag'], 'v0.3.0-unstable')
            self.assertEqual((root / 'landing/src/lib/prompt-data.js').read_text(), 'https://example.com/v0.3.0-unstable/sigma')
            self.assertEqual(release.release(root, write=True)['sha'], result['sha'])
            self.assertFalse(release.release(root)['released'])
            (root / 'source').write_text('dirty')
            with self.assertRaises(ValueError):
                release.release(root, write=True)


if __name__ == '__main__':
    unittest.main()
