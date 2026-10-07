// Opening the browser database, and the `meta` store.

import { type IDBPDatabase, openDB } from "idb";

import { DB_NAME, DB_VERSION, type OffcutDb, STORES, type Versioned } from "./schema";

type Upgrade = (db: IDBPDatabase<OffcutDb>) => void;

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
];

let opened: Promise<IDBPDatabase<OffcutDb>> | undefined;

/** The database, opened once. A failed attempt is forgotten, so a later call tries again. */
export function openDb(): Promise<IDBPDatabase<OffcutDb>> {
  opened ??= open().catch((error: unknown) => {
    opened = undefined;
    throw error;
  });
  return opened;
}

function open(): Promise<IDBPDatabase<OffcutDb>> {
  return openDB<OffcutDb>(DB_NAME, DB_VERSION, {
    upgrade(db, oldVersion) {
      for (const upgrade of UPGRADES.slice(oldVersion)) {
        upgrade(db);
      }
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
