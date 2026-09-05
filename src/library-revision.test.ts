import cargoManifest from '../src-tauri/Cargo.toml?raw';
import cargoLock from '../src-tauri/Cargo.lock?raw';
import { describe, expect, it } from 'vitest';
import { RNGKIT_CORE_REVISION } from './library-revision';

describe('library revision pin', () => {
  it('matches backend dependency pins', () => {
    const manifestRevisions = [
      ...cargoManifest.matchAll(/^rngkit-[^\n]+rev = "([^"]+)"/gm),
    ].map(([, revision]) => revision);
    expect(manifestRevisions.length).toBeGreaterThan(0);
    expect(
      manifestRevisions.every((revision) => revision === RNGKIT_CORE_REVISION),
    ).toBe(true);

    const lockRevisions = [
      ...cargoLock.matchAll(
        /source = "git\+https:\/\/github\.com\/Thiagojm\/rngkit-core\?rev=([^#"]+)#([^"]+)"/g,
      ),
    ].flatMap(([, revision, commit]) => [revision, commit]);
    expect(lockRevisions.length).toBeGreaterThan(0);
    expect(
      lockRevisions.every((revision) => revision === RNGKIT_CORE_REVISION),
    ).toBe(true);
  });
});
