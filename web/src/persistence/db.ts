// Opening the browser database, and the `meta` store.

import { type IDBPDatabase, openDB } from "idb";

import {
  DB_NAME,
  DB_STRUCTURE_VERSION,
  DB_VERSION,
  type OffcutDb,
  type OffcutPlansDb,
  type OffcutSourcesDb,
  PLAN_STORES,
  PLANS_STRUCTURE_VERSION,
  SOURCE_STORES,
  STORES,
  type Versioned,
} from "./schema";

type Upgrade = (db: IDBPDatabase<OffcutSourcesDb>) => void;

/**
 * The upgrade to version `n` is entry `n - 1`. Opening the database runs, in
 * order, every upgrade the stored database has not had yet.
 */
const UPGRADES: readonly Upgrade[] = [
  // 1: all eight stores (D-6). Keys are given on every write.
  (db) => {
    for (const store of Object.values(STORES)) {
      db.createObjectStore(store);
    }
  },
  // 2 (V6.1, D-414): the three source stores, made only when the long path first opens them.
  (db) => {
    for (const store of Object.values(SOURCE_STORES)) {
      db.createObjectStore(store);
    }
  },
  // 3 (V6.2, D-469): the `plans` store, made only when `plans-repo.ts` first opens it.
  // Opened from 1, upgrade 2 runs first, so the source stores exist, empty.
  (db) => {
    (db as unknown as IDBPDatabase<OffcutPlansDb>).createObjectStore(PLAN_STORES.plans);
  },
];

/** The upgrades from `oldVersion` up to the version being opened, and no further. */
function runUpgrades(db: IDBPDatabase<OffcutSourcesDb>, oldVersion: number, newVersion: number | null): void {
  for (const upgrade of UPGRADES.slice(oldVersion, newVersion ?? DB_VERSION)) {
    upgrade(db);
  }
}

function isVersionError(error: unknown): boolean {
  return error instanceof DOMException && error.name === "VersionError";
}

let opened: Promise<IDBPDatabase<OffcutDb>> | undefined;

/** The database, opened once. A failed attempt is forgotten, so a later call tries again. */
export function openDb(): Promise<IDBPDatabase<OffcutDb>> {
  opened ??= open().catch((error: unknown) => {
    opened = undefined;
    throw error;
  });
  return opened;
}

async function open(): Promise<IDBPDatabase<OffcutDb>> {
  try {
    return await openAt(DB_VERSION);
  } catch (error) {
    // V6.1 (D-414, §12.4): the database is newer than this path asks for (the long
    // path upgraded it, or a later build did). Open it as it is; this path uses only
    // the stores it knows, which every later version keeps.
    if (isVersionError(error)) {
      return openAt(undefined);
    }
    throw error;
  }
}

function openAt(version: number | undefined): Promise<IDBPDatabase<OffcutDb>> {
  return openDB<OffcutDb>(DB_NAME, version, {
    upgrade(db, oldVersion, newVersion) {
      runUpgrades(db as unknown as IDBPDatabase<OffcutSourcesDb>, oldVersion, newVersion);
    },
    // A newer build in another tab wants to upgrade: let it.
    blocking(_currentVersion, _blockedVersion, event) {
      (event.target as IDBDatabase).close();
      opened = undefined;
    },
    terminated() {
      opened = undefined;
    },
  });
}

let openedSources: Promise<IDBPDatabase<OffcutSourcesDb>> | undefined;

/**
 * V6.1 (D-414): the database at structure version 2, with the source stores,
 * upgraded on first use. Only `sources-repo.ts` calls this, so a build with
 * `long_video` off never upgrades anyone's database. A connection the clip
 * path holds at version 1 closes itself when asked (`blocking`).
 */
export function openSourcesDb(): Promise<IDBPDatabase<OffcutSourcesDb>> {
  openedSources ??= openSources().catch((error: unknown) => {
    openedSources = undefined;
    throw error;
  });
  return openedSources;
}

async function openSources(): Promise<IDBPDatabase<OffcutSourcesDb>> {
  const at = (version: number | undefined) =>
    openDB<OffcutSourcesDb>(DB_NAME, version, {
      upgrade(db, oldVersion, newVersion) {
        runUpgrades(db, oldVersion, newVersion);
      },
      blocking(_currentVersion, _blockedVersion, event) {
        (event.target as IDBDatabase).close();
        openedSources = undefined;
      },
      terminated() {
        openedSources = undefined;
      },
    });
  try {
    return await at(DB_STRUCTURE_VERSION);
  } catch (error) {
    if (isVersionError(error)) {
      return at(undefined);
    }
    throw error;
  }
}

let openedPlans: Promise<IDBPDatabase<OffcutPlansDb>> | undefined;

/**
 * V6.2 (D-469): the database at structure version 3, with the `plans` store,
 * upgraded on first use, as `openSourcesDb` opens version 2. Only
 * `plans-repo.ts` calls this, and only with `ai_plan` on, so a build with the
 * flag off never upgrades anyone's database. A connection the clip path or the
 * long path holds closes itself when asked (`blocking`); a database newer than
 * 3 is opened as it is (`VersionError`), as V6.1's two openers do.
 */
export function openPlansDb(): Promise<IDBPDatabase<OffcutPlansDb>> {
  openedPlans ??= openPlans().catch((error: unknown) => {
    openedPlans = undefined;
    throw error;
  });
  return openedPlans;
}

async function openPlans(): Promise<IDBPDatabase<OffcutPlansDb>> {
  const at = (version: number | undefined) =>
    openDB<OffcutPlansDb>(DB_NAME, version, {
      upgrade(db, oldVersion, newVersion) {
        runUpgrades(db as unknown as IDBPDatabase<OffcutSourcesDb>, oldVersion, newVersion);
      },
      blocking(_currentVersion, _blockedVersion, event) {
        (event.target as IDBDatabase).close();
        openedPlans = undefined;
      },
      terminated() {
        openedPlans = undefined;
      },
    });
  try {
    return await at(PLANS_STRUCTURE_VERSION);
  } catch (error) {
    if (isVersionError(error)) {
      return at(undefined);
    }
    throw error;
  }
}

/** The value stored under `key`, or `undefined` if there is none. */
export async function metaGet<T>(key: string): Promise<T | undefined> {
  const db = await openDb();
  const stored = await db.get(STORES.meta, key);
  if (stored === undefined) {
    return undefined;
  }
  if (stored.schemaVersion > DB_VERSION) {
    // Written by a newer build: absent to this one, and removed (TS §23.3).
    await db.delete(STORES.meta, key);
    return undefined;
  }
  return stored.value as T;
}

export async function metaSet<T>(key: string, value: T): Promise<void> {
  const db = await openDb();
  const stored: Versioned<T> = { schemaVersion: DB_VERSION, value };
  await db.put(STORES.meta, stored, key);
}

/** V6.2 (D-486): removes what is stored under `key`; nothing stored is not an error. */
export async function metaDelete(key: string): Promise<void> {
  const db = await openDb();
  await db.delete(STORES.meta, key);
}
